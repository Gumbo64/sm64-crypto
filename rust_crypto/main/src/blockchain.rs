use std::u128;

use bytes::Bytes;
use futures_lite::StreamExt;

use n0_future::task;
use n0_snafu::{Result, ResultExt};

use serde::{Deserialize, Serialize};

use snafu::whatever;

mod use_exes;

use use_exes::{record, ez_evaluate, remove_tmp_so_files};
use sha2::{Sha256, Digest};

use distributed_topic_tracker::{AutoDiscoveryGossip, RecordPublisher, TopicId, GossipReceiver, GossipSender};
use iroh_blobs::{api::{ downloader::{Downloader, Shuffled}, tags::Tags }, store::fs::FsStore, BlobsProtocol, Hash };
use iroh::{Endpoint, PublicKey };
use iroh_gossip::{
    api::{Event},
    net::Gossip,
    proto::{DeliveryScope::{Neighbors, Swarm}}
};

use iroh::protocol::Router;
// use iroh_docs::{protocol::Docs};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::blockchain::use_exes::record_loop;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockHead {
    hash: Hash,
    height: u128,
}
impl BlockHead {
    fn no_blocks(&self) -> bool {
        let def = BlockHead::default();
        self.hash == def.hash && self.height == def.height
    }

    fn encode(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(&self).e()?.into())
    }

    pub fn decode(bytes: &[u8]) -> Result<BlockHead> {
        Ok(postcard::from_bytes(bytes).e()?)
    }
    
}
impl Default for BlockHead {
    fn default() -> Self {
        BlockHead {
            hash: Hash::EMPTY,
            height: u128::MAX,
        }
    }
}

#[derive(Debug)]
pub struct BlockChain {
    router: Router, downloader: Downloader, blobs: BlobsProtocol, tags: Tags,
    sender: GossipSender, db_lock: Arc<Mutex<()>>, new_block_signal: Arc<Mutex<bool>>
}
impl Clone for BlockChain {
    fn clone(&self) -> Self {
        BlockChain {
            router: self.router.clone(),
            downloader: self.downloader.clone(), 
            blobs: self.blobs.clone(),
            tags: self.tags.clone(),
            sender: self.sender.clone(),
            db_lock: Arc::clone(&self.db_lock), // We need to make sure it uses this function not just .clone()
            new_block_signal: Arc::clone(&self.new_block_signal)
        }
    }
}

impl BlockChain {
    pub async fn new(nowait: bool) -> Result<BlockChain> {
        remove_tmp_so_files(".").e()?;

        let endpoint = Endpoint::builder()
            .discovery_n0()
            .bind()
            .await?;

        let store_path = String::from("blockchain");

        let store = FsStore::load(store_path).await.expect("failed to load fs");

        // let store = MemStore::new();
        let blobs = BlobsProtocol::new(&store, endpoint.clone(), None);
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let tags = blobs.tags().clone();
        let downloader = store.downloader(&endpoint);

        // Setup router
        let router = Router::builder(endpoint.clone())
            .accept(iroh_blobs::ALPN, blobs.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        // [Distributed Topic Tracker]
        let topic_id = TopicId::new("sm64-crypto".to_string());
        let initial_secret = b"googoo gaga".to_vec();
        let record_publisher = RecordPublisher::new(
            topic_id.clone(),
            endpoint.node_id(),
            endpoint.secret_key().secret().clone(),
            None,
            initial_secret,
        );

        // A new field "subscribe_and_join_with_auto_discovery/_no_wait" 
        // is available on iroh_gossip::net::Gossip
        let topic;
        if !nowait {
            topic = gossip
                .subscribe_and_join_with_auto_discovery(record_publisher)
                .await.expect("Topic subscription failed");
        } else {
            topic = gossip
                .subscribe_and_join_with_auto_discovery_no_wait(record_publisher)
                .await.expect("Topic subscription failed");
        }

        let (sender, receiver)  = topic.split().await.expect("topic split failed");
        println!("\n[Found peers]\n");

        let db_lock = Arc::new(Mutex::new(()));
        let new_block_signal = Arc::new(Mutex::new(false));
        let bc = BlockChain{router, downloader, blobs, tags, sender, db_lock, new_block_signal};
        task::spawn(BlockChain::subscribe_loop(bc.clone(), receiver));
        Ok(bc)
    }

    async fn subscribe_loop(
        bc: BlockChain, receiver: GossipReceiver
    ) -> Result<()> {
        {
            let _guard = bc.db_lock.lock().await;
            bc.request_head().await?;
            bc.broadcast_head().await?;
        }

        bc.print_state().await?;
        while let Some(e) = receiver.next().await {
            if e.is_err() {
                println!("error receiver");
                continue;
            }
            let event = e?;
            let _guard = bc.db_lock.lock().await;
            let res: Result<()> = {
                if let Event::Received(msg) = event {
                    match msg.scope {
                        Neighbors => {} // Only accept direct neighbour messages
                        Swarm(_) => {continue;}
                    }
                    let message = BlockMessage::decode(&msg.content)?;
                    match message {
                        BlockMessage::NewBlockHead { hash } => {
                            let peer_vec: Vec<PublicKey> = receiver.neighbors().await.into_iter().collect();   
                            match bc.new_block(hash, peer_vec.clone()).await {
                                Ok(_) => {
                                    bc.broadcast_head().await?;
                                    bc.print_state().await?;
                                },
                                Err(_) => {println!("New block failed")}
                            }
                        },
                        BlockMessage::RequestBlockHead {} => {
                            bc.broadcast_head().await?;
                        }
                    }
                }
                Ok(())
            };
            match res {
                Ok(_) => continue,
                Err(s) => println!("\n\n\nMISC SUBSCRIBER ERROR {:?}\n\n", s)
            }
        }
        Ok(())
    }
    pub async fn print_state(&self) -> Result<()> {
        let head = self.get_head().await?;
        if !head.no_blocks() {
            println!("Current height: {}\nCurrent hash: {:?}", head.height, head.hash);
        } else {
            println!("\n\nNEW CHAIN\n\n");
        }
        Ok(())
    }

    pub async fn get_head(&self) -> Result<BlockHead> {
        let ott = self.tags.get(String::from("head")).await;

        match ott {
            Ok(ot) => {
                match ot {
                    Some(t) => {
                        let bytes = self.blobs.get_bytes(t.hash).await?;
                        return Ok(BlockHead::decode(&bytes)?);
                    },
                    None => {
                        self.set_head(BlockHead::default()).await?;
                        return Ok(BlockHead::default());
                    }
                }
            },
            Err(_e) => {
                self.set_head(BlockHead::default()).await?;
                return Ok(BlockHead::default());
            }
        }

    }
    async fn set_head(&self, head: BlockHead) -> Result<()> {
        let h = self.blobs.add_bytes(head.encode()?).await?.hash;
        self.tags.set( String::from("head"), h).await?;
        Ok(())
    }

    // Block that might be foreign
    async fn get_foreign_block(&self, hash: Hash, peers: Vec<PublicKey>) -> Result<Block> {
        match self.get_local_block(hash).await {
            Ok(b) => Ok(b), // We have it locally
            Err(_e) => { // Try to get it from peers
                let s_peers = Shuffled::new(peers);

                let mut progress = self.downloader.download(hash, s_peers)
                    .stream().await.e()?;

                while let Some(_event) = progress.next().await {}
                match self.get_local_block(hash).await {
                    Ok(b) => Ok(b),
                    Err(e) => {
                        return Err(e);
                    } 
                }
            }
        }
    }

    async fn get_local_block(&self, hash: Hash) -> Result<Block> {
        let block_bytes = self.blobs.get_bytes(hash).await?;
        Ok(Block::decode(&block_bytes)?)
    }

    async fn hash_at_height(&self, height: u128) -> Option<Hash>{
        Some(self.tags.get(height.to_string()).await.ok()??.hash)
    }

    async fn add_block_blob(&self, block: Block) -> Result<Hash> {
        let encoding = block.encode().e()?;
        let hash = self.blobs.add_bytes(encoding).await.e()?.hash;
        Ok(hash)
    }

    async fn temp_add_block(&self, hash: Hash) -> Result<()> {
        let block = self.get_local_block(hash).await.e()?;
        self.tags.set( String::from("temp_") + &block.block_height.to_string(), hash).await?;
        Ok(())
    }

    async fn clear_temp_blocks(&self) -> Result<()> {
        self.tags.delete_prefix(String::from("temp_")).await.e()?;
        Ok(())
    }

    async fn confirm_block(&self, height: u128) -> Result<()> {
        self.tags.rename(String::from("temp_") + &height.to_string(), height.to_string()).await?;
        Ok(())
    }

    async fn new_block(&self, new_head_hash: Hash, peers: Vec<PublicKey>) -> Result<()> {
        let new_head = self.get_foreign_block(new_head_hash, peers.clone()).await.e()?;

        // check that the new block is even worth it, it should be higher than our head
        let head = self.get_head().await.e()?;
        if !head.no_blocks() && (new_head.block_height <= head.height) {
            whatever!("New head is worse than the old one");
        }

        // Loop from the head downwards, validating all those blocks
        let mut cur_hash = new_head_hash.clone();
        let mut cur_height = new_head.block_height.clone();
        let mut block = new_head.clone();
        loop {

            if let Some(h) = self.hash_at_height(cur_height).await {
                if h == cur_hash {
                    // We already have this block in our chain, so we can stop
                    break;
                }
            }
            block = self.get_foreign_block(cur_hash, peers.clone()).await.e()?;

            if cur_height != block.block_height {
                whatever!("Non sequential blocks");
            }
            if cur_height == 0 && block.prev_hash != Hash::EMPTY {
                whatever!("Failed genesis block");
            }

            // Test if it works
            if !block.evaluate_replay().await {
                whatever!("Replay fail");
            }

            // Add this block to the temporary storage
            self.temp_add_block(cur_hash).await?;

            // We have reached the genesis block
            if cur_height == 0 {
                break;
            }
            cur_hash = block.prev_hash;
            cur_height -= 1;
        }

        // Once all blocks are validated and stored in the temporary area, update it to be our new blockchain
        for i in (block.block_height)..(new_head.block_height) {
            self.confirm_block(i).await?;
        }

        let new_blockhead = BlockHead {hash: new_head_hash, height: new_head.block_height };
        self.set_head(new_blockhead).await.e()?;
        self.clear_temp_blocks().await?;

        let mut new_block_signal = self.new_block_signal.lock().await;
        *new_block_signal = true;

        Ok(())
    }

    async fn broadcast_block(&self, hash: Hash) -> Result<()> {
        let message = BlockMessage::NewBlockHead { hash };
        let encoded = message.encode().e()?.to_vec();
        // sender.broadcast_neighbors(encoded).await.e()?;
        match self.sender.broadcast_neighbors(encoded).await {
            Ok(_) => Ok(()),
            Err(_) => whatever!("Broadcast block fail") // Convert the error
        }
    }

    async fn broadcast_head(&self) -> Result<()> {
        let head = self.get_head().await.e()?;
        if head.no_blocks(){
            return Ok(());
        }
        self.broadcast_block(head.hash).await
    }
    async fn request_head(&self) -> Result<()> {
        let message = BlockMessage::RequestBlockHead{};
        let encoded = message.encode().e()?.to_vec();
        match self.sender.broadcast_neighbors(encoded).await {
            Ok(_) => Ok(()),
            Err(_) => whatever!("Request head fail") // Convert the error
        }
    }
    pub async fn mine(&self) {
        loop {
            match self.mine_attempt().await {
                Ok(_) => {
                    return;
                },
                Err(e) => {
                    println!("Resetting the game so we're mining from the newest block");
                }
            }
        }
    }
    async fn mine_attempt(&self) -> Result<()> {
        let mut _guard = self.db_lock.lock().await;

        {
            // If any new blocks are made while we're playing, then it will kill the recording session
            let mut new_block_signal = self.new_block_signal.lock().await;
            *new_block_signal = false;
        }

        let head = self.get_head().await.e()?;

        // Mine a block. Release and then retake the lock after you finish playing
        drop(_guard);
        let new_block = Block::new(head, self.new_block_signal.clone()).e()?;
        _guard = self.db_lock.lock().await;

        // Add block to blobs
        let new_hash = self.add_block_blob(new_block).await.e()?;

        // You never have to download anything since you mined it locally, therefore no peers are needed
        match self.new_block(new_hash, vec![]).await {
            Ok(_) => {
                self.broadcast_block(new_hash).await?;
                self.print_state().await?;
            },
            Err(_) => {println!("New block failed")}
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Block {
    prev_hash: Hash,
    block_height: u128,
    solution_bytes: Vec<u8>,
}

impl Block {
    pub fn new(block_head: BlockHead, kill_signal: Arc<Mutex<bool>>) -> Result<Self> {
        let prev_hash = block_head.hash;
        let block_height = block_head.height.wrapping_add(1);

        let seed = Block {prev_hash, block_height, solution_bytes: Vec::new() }.calc_seed();
        // let solution_bytes = ez_record_loop(&seed);
        // if solution_bytes.len() == 0 {
            // whatever!("Failed to mine");
        // }
        
        let solution_bytes = record_loop(&seed, kill_signal)?;



        Ok(Block { prev_hash, block_height, solution_bytes })
    }
    async fn evaluate_replay(&self) -> bool {
        ez_evaluate(&self.calc_seed(), &self.solution_bytes, 0)
    }
    fn calc_seed(&self) -> String {
        let combined = format!("{}{}", self.prev_hash, self.block_height);
        let mut hasher = Sha256::new();
        hasher.update(combined);
        let result = hasher.finalize();
        hex::encode(result) // Convert the hash to a hex string
    }

    fn encode(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(&self).e()?.into())
    }

    pub fn decode(bytes: &[u8]) -> Result<Block> {
        Ok(postcard::from_bytes(bytes).e()?)
    }

}

#[derive(Debug, Serialize, Deserialize)]
enum BlockMessage {
    NewBlockHead { hash: Hash },
    RequestBlockHead { },
}

impl BlockMessage {
    pub fn decode(bytes: &[u8]) -> Result<BlockMessage> {
        Ok(postcard::from_bytes(bytes).e()?)
    }

    fn encode(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(&self).e()?.into())
    }
}
