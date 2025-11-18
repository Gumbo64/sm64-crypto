use std::u128;
use bytes::Bytes;
use futures_lite::StreamExt;
use iroh_blobs::api::Store;
#[cfg(not(feature = "fs"))]
use iroh_blobs::store::mem::MemStore;
use n0_future::task;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

// use distributed_topic_tracker::{AutoDiscoveryGossip, RecordPublisher, TopicId, GossipReceiver, GossipSender};
// use mainline::SigningKey;

use iroh_blobs::{api::{ downloader::{Downloader, Shuffled}, tags::Tags }, BlobsProtocol, Hash };
use iroh::{Endpoint, EndpointId, PublicKey };
use iroh_gossip::{
    api::{Event, GossipReceiver, GossipSender}, net::Gossip, proto::DeliveryScope::{Neighbors, Swarm}
};

use iroh::protocol::Router;
use tracing::info;
// use iroh_docs::{protocol::Docs};
// use std::sync::{Arc, Mutex};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Local};

use crate::DEFAULT_CONFIG;

pub use sm64_binds::{GamePad, RandomConfig, SM64GameGenerator};

mod block;
mod ticket;

pub use block::{BlockHead, Block};
pub use ticket::Ticket;



#[derive(Debug)]
pub struct BlockChain {
    router: Router, downloader: Downloader, blobs: BlobsProtocol, tags: Tags, sender: GossipSender, game_gen: SM64GameGenerator, random_config: RandomConfig,
    db_lock: Arc<Mutex<()>>, new_block_signal: Arc<Mutex<bool>>, eval_request: Arc<Mutex<Option<(Block, Option<bool>)>>>,
}
impl Clone for BlockChain {
    fn clone(&self) -> Self {
        BlockChain {
            router: self.router.clone(),
            downloader: self.downloader.clone(), 
            blobs: self.blobs.clone(),
            tags: self.tags.clone(),
            sender: self.sender.clone(),
            game_gen: self.game_gen.clone(),
            random_config: self.random_config.clone(),
            db_lock: Arc::clone(&self.db_lock), // We need to make sure it uses this function not just .clone()
            new_block_signal: Arc::clone(&self.new_block_signal),
            eval_request: Arc::clone(&self.eval_request),
        }
    }
}

impl BlockChain {
    pub async fn new(game_gen: SM64GameGenerator, ticket: Ticket) -> Result<Self> {
        info!("CCCCC");
        let endpoint = Endpoint::builder().bind().await?;

        let store = load_store().await;

        let blobs = BlobsProtocol::new(&store, None);
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let tags = blobs.tags().clone();
        let downloader = store.downloader(&endpoint);

        // Setup router
        let router = Router::builder(endpoint.clone())
            .accept(iroh_blobs::ALPN, blobs.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();


        let db_lock = Arc::new(Mutex::new(()));
        let new_block_signal = Arc::new(Mutex::new(false));
        let eval_request = Arc::new(Mutex::new(None));
        let random_config = RandomConfig::default();

        let topic_id = ticket.topic_id;
        let bootstrap = ticket.bootstrap.iter().cloned().collect();
        let (sender, receiver) = gossip.subscribe(topic_id, bootstrap).await?.split();

        let bc = BlockChain {router, downloader, blobs, tags, sender, game_gen, random_config, db_lock, new_block_signal, eval_request};
        let bc2 = bc.clone();
        task::spawn(async move {
            let result = BlockChain::subscribe_loop(bc2, receiver).await;

            match result {
                Ok(_) => info!("Subscribe loop finished successfully."),
                Err(e) => info!("Subscribe loop finished with error: {:?}", e),
            }
        });
        info!("CCCCC");

        Ok(bc)
    }

    pub fn endpoint_id(&self) -> EndpointId {
        self.router.endpoint().id()
    }

    async fn subscribe_loop(
        bc: BlockChain, mut receiver: GossipReceiver
    ) -> Result<()> {
        info!("SUBLOOP: Waiting to connect...");
        receiver.joined().await?;
        info!("SUBLOOP: Connected!");
        {
            let _guard = bc.db_lock.lock().await;
            bc.request_head().await?;
            bc.broadcast_head().await?;
        }

        bc.print_state().await?;
        while let Some(e) = receiver.next().await {
            if e.is_err() {
                info!("error receiver");
                continue;
            }
            let event = e?;
            let _guard = bc.db_lock.lock().await;
            let res: Result<()> = {
                if let Event::Received(msg) = event {
                    match msg.scope {
                        Neighbors => {} // Only accept direct neighbour messages
                        Swarm(_) => {info!("Bad message scope"); continue;}
                    }
                    let message = BlockMessage::decode(&msg.content)?;
                    match message {
                        BlockMessage::NewBlockHead { hash } => {
                            let peer_vec: Vec<PublicKey> = receiver.neighbors().into_iter().collect();   
                            match bc.new_block(hash, peer_vec.clone()).await {
                                Ok(_) => {
                                    bc.broadcast_head().await?;
                                    bc.print_state().await?;
                                },
                                Err(_) => {info!("New block failed")}
                            }
                        },
                        BlockMessage::RequestBlockHead {} => {
                            bc.broadcast_head().await?;
                        }
                    }
                }
                else if let Event::NeighborUp(key) = event {info!("Joined {}", key);}
                else if let Event::NeighborDown(key) = event {info!("Downed {}", key);}
                else if let Event::Lagged = event {info!("Lagged");}
                Ok(())
            };
            match res {
                Ok(_) => continue,
                Err(s) => info!("\n\n\nMISC SUBSCRIBER ERROR {:?}\n\n", s)
            }
        }
        Ok(())
    }
    async fn print_state(&self) -> Result<()> {
        let head = self.get_head().await?;
        if !head.no_blocks() {
            let head_block = self.get_local_block(head.hash).await?;

            let name = match String::from_utf8(head_block.miner_name.to_vec()) {
                Ok(string) => {
                    let trim_str = string.trim_end_matches('\0').to_string();
                    match trim_str.is_ascii() {
                        true => {
                            trim_str
                        }
                        false => {
                            String::from("Unknown")
                        }
                    }
                }
                Err(_e) => {
                    String::from("Unknown")
                }
            };
            

            let converted_timestamp: DateTime<Local> = DateTime::from(head_block.timestamp);
            info!("Current height: {}\nCurrent hash: {:?}\nAt time: {:?}\nMined by: {:?}", head.height, head.hash, converted_timestamp, name);
        } else {
            info!("\n\nNEW CHAIN\n\n");
        }
        Ok(())
    }

    async fn get_head(&self) -> Result<BlockHead> {
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
                    .stream().await?;

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
        let encoding = block.encode()?;
        let hash = self.blobs.add_bytes(encoding).await?.hash;
        Ok(hash)
    }

    async fn temp_add_block(&self, hash: Hash) -> Result<()> {
        let block = self.get_local_block(hash).await?;
        self.tags.set( String::from("temp_") + &block.block_height.to_string(), hash).await?;
        Ok(())
    }

    async fn clear_temp_blocks(&self) -> Result<()> {
        self.tags.delete_prefix(String::from("temp_")).await?;
        Ok(())
    }

    async fn confirm_block(&self, height: u128) -> Result<()> {
        self.tags.rename(String::from("temp_") + &height.to_string(), height.to_string()).await?;
        Ok(())
    }

    async fn new_block(&self, new_head_hash: Hash, peers: Vec<PublicKey>) -> Result<()> {
        let new_head = self.get_foreign_block(new_head_hash, peers.clone()).await?;

        // check that the new block is even worth it, it should be higher than our head
        let head = self.get_head().await?;
        if !head.no_blocks() && (new_head.block_height <= head.height) {
            return Err(Error::msg("New head is worse than the old one"));
        }

        // Loop from the head downwards, validating all those blocks
        let mut cur_hash = new_head_hash.clone();
        let mut cur_height = new_head.block_height.clone();
        let mut block = new_head.clone();
        loop {
            // Check block
            if let Some(h) = self.hash_at_height(cur_height).await {
                if h == cur_hash {
                    // We already have this block in our chain, so we can stop
                    break;
                }
            }
            block = self.get_foreign_block(cur_hash, peers.clone()).await?;

            if cur_height != block.block_height {
                return Err(Error::msg("Non sequential blocks"));
            }
            if cur_height == 0 && block.prev_hash != Hash::EMPTY {
                return Err(Error::msg("Failed genesis block"));
            }

            // Check replay
            if !self.evaluate_replay(block).await? {
                return Err(Error::msg("Replay fail"));
            }

            // block is verified, add to the temporary storage, wait for lower blocks to be confirmed
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
        self.set_head(new_blockhead).await?;
        self.clear_temp_blocks().await?;

        let mut new_block_signal = self.new_block_signal.lock().await;
        *new_block_signal = true;

        Ok(())
    }

    async fn evaluate_replay(&self, block: Block) -> Result<bool> {
        let mut game = self.game_gen.create_game()?;
        game.rng_init(block.calc_seed(), self.random_config)?;

        for p_pad in block.get_solution().iter() {
            let pad = *p_pad;
            let random_pad = game.rng_pad(pad)?;

            if !pad.equals(&random_pad) {
                return Ok(false);
            }

            game.step_game(pad)?;
            let state = game.get_game_state()?;

            if state.has_won() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn broadcast_block(&self, hash: Hash) -> Result<()> {
        let message = BlockMessage::NewBlockHead { hash };
        let encoded = message.encode()?.to_vec();
        // sender.broadcast_neighbors(encoded).await?;
        match self.sender.broadcast_neighbors(encoded.into()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::msg("Broadcast block failed")),
        }
    }

    async fn broadcast_head(&self) -> Result<()> {
        let head = self.get_head().await?;
        if head.no_blocks(){
            return Ok(());
        }
        self.broadcast_block(head.hash).await
    }

    async fn request_head(&self) -> Result<()> {
        let message = BlockMessage::RequestBlockHead{};
        let encoded = message.encode()?.to_vec();
        match self.sender.broadcast_neighbors(encoded.into()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::msg("Request head failed")),
        }
    }

    pub async fn start_mine(&self, miner_name: [u8; DEFAULT_CONFIG.max_name_length]) -> Result<Block> {
        let mut _guard = self.db_lock.lock().await;
        {
            // If any new blocks are made while we're playing, then it will kill the recording session
            let mut new_block_signal = self.new_block_signal.lock().await;
            *new_block_signal = false;
        }

        let head = self.get_head().await?;

        Ok(Block::new(head, miner_name))
    }

    pub async fn submit_mine(&self, new_block: Block) -> Result<()> {
        let _guard = self.db_lock.lock().await;

        // Add block to blobs
        let new_hash = self.add_block_blob(new_block).await?;

        // You never have to download anything since you mined it locally, therefore no peers are needed
        match self.new_block(new_hash, vec![]).await {
            Ok(_) => {
                self.broadcast_block(new_hash).await?;
                self.print_state().await?;
            },
            Err(_) => {info!("New block failed")}
        }
        Ok(())
    }

    pub async fn has_new_block(&self) -> bool {
        // Return true if there is a new block (aka the head has been updated)
        let mut nb_p = self.new_block_signal.lock().await;
        if *nb_p {
            *nb_p = false;
            return true;
        }
        false
    }
    // these two have _public because we shouldn't use them in this file. because we don't want to acquire locks
    pub async fn get_head_block_public(&self) -> Result<Block> {
        let _guard = self.db_lock.lock().await;
        let head = self.get_head().await?;
        self.get_local_block(head.hash).await
    }

    pub async fn get_local_block_public(&self, hash: Hash) -> Result<Block> {
        let mut _guard = self.db_lock.lock().await;
        self.get_local_block(hash).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum BlockMessage {
    NewBlockHead { hash: Hash },
    RequestBlockHead { },
}

impl BlockMessage {
    pub fn decode(bytes: &[u8]) -> Result<BlockMessage> {
        Ok(postcard::from_bytes(bytes)?)
    }

    fn encode(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(&self)?.into())
    }
}

#[cfg(feature = "fs")]
async fn load_store() -> Store {
    let store_path = String::from("blockchain_data");
    iroh_blobs::store::fs::FsStore::load(store_path)
        .await
        .expect("failed to load fs").into()
}

#[cfg(not(feature = "fs"))]
async fn load_store() -> Store {
    MemStore::new().into()
}