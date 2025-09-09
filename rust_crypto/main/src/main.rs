use std::u128;

use bytes::Bytes;
use clap::Parser;
use futures_lite::StreamExt;

use n0_future::task;
use n0_snafu::{Result, ResultExt};

use serde::{Deserialize, Serialize};

use snafu::whatever;

mod use_exes;

use use_exes::{ez_record_loop, ez_evaluate, remove_tmp_so_files};
use sha2::{Sha256, Digest};

use distributed_topic_tracker::{AutoDiscoveryGossip, RecordPublisher, TopicId, GossipReceiver, GossipSender};
use iroh_blobs::{api::{ downloader::{Downloader, Shuffled}, tags::Tags }, store::fs::FsStore, BlobsProtocol, Hash };
use iroh::{Endpoint, PublicKey };
use iroh_gossip::{
    api::{Event},
    net::Gossip,
    proto::{DeliveryScope::{Neighbors, Swarm}}
};
use std::sync::atomic::{AtomicBool, Ordering};
use iroh::protocol::Router;
// use iroh_docs::{protocol::Docs};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
struct Args {
    /// Enable mining
    #[clap(short, long, default_value_t = false)]
    mine: bool,
    /// Wait for a connection before starting
    #[clap(short, long, default_value_t = false)]
    nowait: bool,
}

#[derive(Parser, Debug)]
enum Command {
    /// Open a chat room for a topic and print a ticket for others to join.
    ///
    /// If no topic is provided, a new topic will be created.
    Open,
    /// Join a chat room from a ticket.
    Join {
        /// The ticket, as base32 string.
        ticket: String,
    },
}


#[derive(Debug, Serialize, Deserialize, Clone)]
struct BlockHead {
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

#[derive(Debug, Clone)]
struct BlockChain {}
impl BlockChain {
    async fn print_state(&self, blobs: &BlobsProtocol, tags: &Tags) -> Result<()> {
        let head = self.get_head(blobs, tags).await?;
        if !head.no_blocks() {
            println!("Current height: {}\nCurrent hash: {:?}", head.height, head.hash);
        } else {
            println!("\n\nNEW CHAIN\n\n");
        }
        Ok(())
    }

    async fn get_head(&self, blobs: &BlobsProtocol, tags: &Tags) -> Result<BlockHead> {
        let ott = tags.get(String::from("head")).await;

        match ott {
            Ok(ot) => {
                match ot {
                    Some(t) => {
                        let bytes = blobs.get_bytes(t.hash).await?;
                        return Ok(BlockHead::decode(&bytes)?);
                    },
                    None => {
                        self.set_head(blobs, tags, BlockHead::default()).await?;
                        return Ok(BlockHead::default());
                    }
                }
            },
            Err(_e) => {
                self.set_head(blobs, tags, BlockHead::default()).await?;
                return Ok(BlockHead::default());
            }
        }

    }
    async fn set_head(&self, blobs: &BlobsProtocol, tags: &Tags, head: BlockHead) -> Result<()> {
        let h = blobs.add_bytes(head.encode()?).await?.hash;
        tags.set( String::from("head"), h).await?;
        Ok(())
    }

    // Block that might be foreign
    async fn get_foreign_block(&self, blobs: &BlobsProtocol, downloader: &Downloader, hash: Hash, peers: Vec<PublicKey>) -> Result<Block> {
        match self.get_local_block(blobs, hash).await {
            Ok(b) => Ok(b), // We have it locally
            Err(_e) => { // Try to get it from peers
                let s_peers = Shuffled::new(peers);
                let mut progress = downloader.download(hash, s_peers)
                    .stream().await.e()?;

                while let Some(_event) = progress.next().await {}

                Ok(self.get_local_block(blobs, hash).await?)
            }
        }
    }

    async fn get_local_block(&self, blobs: &BlobsProtocol, hash: Hash) -> Result<Block> {
        let block_bytes = blobs.get_bytes(hash).await?;
        Ok(Block::decode(&block_bytes)?)
    }

    async fn hash_at_height(&self, tags: &Tags, height: u128) -> Option<Hash>{
        Some(tags.get(height.to_string()).await.ok()??.hash)
    }

    async fn add_block_blob(&self, blobs: &BlobsProtocol, block: Block) -> Result<Hash> {
        let encoding = block.encode().e()?;
        let hash = blobs.add_bytes(encoding).await.e()?.hash;
        Ok(hash)
    }

    async fn temp_add_block(&self, blobs: &BlobsProtocol, tags: &Tags, hash: Hash) -> Result<()> {
        let block = self.get_local_block(blobs, hash).await.e()?;
        tags.set( String::from("temp_") + &block.block_height.to_string(), hash).await?;
        Ok(())
    }

    async fn clear_temp_blocks(&self, tags: &Tags) -> Result<()> {
        tags.delete_prefix(String::from("temp_")).await.e()?;
        Ok(())
    }

    async fn confirm_block(&self, tags: &Tags, height: u128) -> Result<()> {
        tags.rename(String::from("temp_") + &height.to_string(), height.to_string()).await?;
        Ok(())
    }

    async fn new_block(&self, blobs: &BlobsProtocol, tags: &Tags, downloader: &Downloader, new_head_hash: Hash, peers: Vec<PublicKey>) -> Result<()> {

        let new_head = self.get_foreign_block(blobs, downloader, new_head_hash, peers.clone()).await.e()?;

        // check that the new block is even worth it, it should be higher than our head
        let head = self.get_head(blobs, tags).await.e()?;
        if !head.no_blocks() && (new_head.block_height <= head.height) {
            whatever!("New head is worse than the old one");
        }

        // Loop from the head downwards, validating all those blocks
        let mut cur_hash = new_head_hash.clone();
        let mut cur_height = new_head.block_height.clone();
        let mut block = new_head.clone();
        loop {

            if let Some(h) = self.hash_at_height(tags, cur_height).await {
                if h == cur_hash {
                    // We already have this block in our chain, so we can stop
                    break;
                }
            }
            block = self.get_foreign_block(blobs, downloader, cur_hash, peers.clone()).await.e()?;

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
            self.temp_add_block(blobs, tags, cur_hash).await?;

            // We have reached the genesis block
            if cur_height == 0 {
                break;
            }
            cur_hash = block.prev_hash;
            cur_height -= 1;
        }

        // Once all blocks are validated and stored in the temporary area, update it to be our new blockchain
        for i in (block.block_height)..(new_head.block_height) {
            self.confirm_block(tags, i).await?;
        }

        let new_blockhead = BlockHead {hash: new_head_hash, height: new_head.block_height };
        self.set_head(blobs, tags, new_blockhead).await.e()?;
        self.clear_temp_blocks(tags).await?;

        Ok(())
    }

    async fn broadcast_block(&self, sender: &GossipSender, hash: Hash) -> Result<()> {
        let message = BlockMessage::NewBlockHead { hash };
        let encoded = message.encode().e()?.to_vec();
        // sender.broadcast_neighbors(encoded).await.e()?;
        match sender.broadcast_neighbors(encoded).await {
            Ok(_) => Ok(()),
            Err(_) => whatever!("Broadcast block fail") // Convert the error
        }
    }

    async fn broadcast_head(&self, blobs: &BlobsProtocol, tags: &Tags, sender: &GossipSender) -> Result<()> {
        let head = self.get_head(blobs, tags).await.e()?;
        if head.no_blocks(){
            return Ok(());
        }
        self.broadcast_block(sender, head.hash).await
    }
    async fn request_head(&self, sender: &GossipSender) -> Result<()> {
        let message = BlockMessage::RequestBlockHead{};
        let encoded = message.encode().e()?.to_vec();
        match sender.broadcast_neighbors(encoded).await {
            Ok(_) => Ok(()),
            Err(_) => whatever!("Request head fail") // Convert the error
        }
    }


}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Block {
    prev_hash: Hash,
    block_height: u128,
    solution_bytes: Vec<u8>,
}

impl Block {
    pub fn new(block_head: BlockHead) -> Result<Self> {
        let prev_hash = block_head.hash;
        let block_height = block_head.height.wrapping_add(1);

        let seed = Block {prev_hash, block_height, solution_bytes: Vec::new() }.calc_seed();
        let solution_bytes = ez_record_loop(&seed);
        if solution_bytes.len() == 0 {
            whatever!("Failed to mine");
        }

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

async fn subscribe_loop(
    downloader: Downloader, blobs: BlobsProtocol, tags: Tags,
    receiver: GossipReceiver, sender: GossipSender, 
    db_lock: Arc<Mutex<()>>
) -> Result<()> {

    let bc = BlockChain{};
    {
        let _guard = db_lock.lock().await;
        bc.request_head(&sender).await?;
        bc.broadcast_head(&blobs, &tags, &sender).await?;
    }

    bc.print_state(&blobs, &tags).await?;
    while let Some(e) = receiver.next().await {
        if e.is_err() {
            println!("error receiver");
            continue;
        }
        let event = e?;
        let _guard = db_lock.lock().await;
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

                        match bc.new_block(&blobs, &tags, &downloader, hash, peer_vec.clone()).await {
                            Ok(_) => {
                                bc.broadcast_head(&blobs, &tags, &sender).await?;
                                bc.print_state(&blobs, &tags).await?;
                            },
                            Err(_) => {println!("New block failed")}
                        }
                    },
                    BlockMessage::RequestBlockHead {} => {
                        bc.broadcast_head(&blobs, &tags, &sender).await?;
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


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    remove_tmp_so_files(".").e()?;

    println!("mine {}, nowait {}", args.mine, args.nowait);
    
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");


    let endpoint = Endpoint::builder()
        .discovery_n0()
        .bind()
        .await?;

    // Create protocols

    // let mut rng = rand::rng();
    // let store_path = format!("my_blockchain_{}{}{}", rng.sample(rand::distr::Alphanumeric) as char, rng.sample(rand::distr::Alphanumeric) as char, rng.sample(rand::distr::Alphanumeric) as char);
    let store_path = String::from("blockchain");

    let store = FsStore::load(store_path).await.expect("failed to load fs");

    // let store = MemStore::new();
    let blobs = BlobsProtocol::new(&store, endpoint.clone(), None);
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let tags = blobs.tags();
    let downloader = store.downloader(&endpoint);

    // Setup router
    let _router = Router::builder(endpoint.clone())
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
    if !args.nowait {
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

    task::spawn(subscribe_loop(downloader.clone(), blobs.clone(), tags.clone(), receiver, sender.clone(),Arc::clone(&db_lock)));

    let bc = BlockChain{};

    while running.load(Ordering::SeqCst) {

        if args.mine {
            let mut _guard = db_lock.lock().await;

            let head = bc.get_head(&blobs, tags).await.e()?;

            // Mine a block. Release and then retake the lock after you finish playing
            drop(_guard);
            let new_block = Block::new(head).e()?;
            _guard = db_lock.lock().await;

            // Add block to blobs
            let new_hash = bc.add_block_blob(&blobs, new_block).await.e()?;

            // You never have to download anything since you mined it locally, therefore no peers are needed
            match bc.new_block(&blobs, tags, &downloader, new_hash, vec![]).await {
                Ok(_) => {
                    bc.broadcast_block(&sender, new_hash).await?;
                    bc.print_state(&blobs, &tags).await?;
                },
                Err(_) => {println!("New block failed")}
            }

        }

    }
    println!("Ending the program");
    Ok(())
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
