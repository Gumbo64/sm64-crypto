use std::u128;
use bytes::Bytes;
use futures_lite::StreamExt;
use iroh_blobs::api::Store;
#[cfg(not(feature = "fs"))]
use iroh_blobs::store::mem::MemStore;
use n0_future::task;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Local};

// use distributed_topic_tracker::{AutoDiscoveryGossip, RecordPublisher, TopicId, GossipReceiver, GossipSender};
// use mainline::SigningKey;

use iroh_blobs::{api::{ downloader::{Downloader, Shuffled}, tags::Tags }, BlobsProtocol, Hash };
use iroh::{Endpoint, EndpointId };
use iroh_gossip::{
    api::{Event, GossipReceiver, GossipSender}, net::Gossip,
    // proto::DeliveryScope::{Neighbors, Swarm}
};

use iroh::protocol::Router;
use tracing::info;
// use iroh_docs::{protocol::Docs};
// use std::sync::{Arc, Mutex};
use std::sync::Arc;
use tokio::sync::Mutex;

pub use sm64_binds::{GamePad, RandomConfig, SM64GameGenerator};

mod block;
mod ticket;

pub use block::{BlockHead, Block};
pub use ticket::Ticket;



#[derive(Debug)]
pub struct BlockChain {
    router: Router, downloader: Downloader, blobs: BlobsProtocol, tags: Tags, sender: GossipSender, game_gen: SM64GameGenerator, random_config: RandomConfig,
    db_lock: Arc<Mutex<()>>, new_block_signal: Arc<Mutex<bool>>,
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
        }
    }
}

impl BlockChain {
    pub async fn new(game_gen: SM64GameGenerator, ticket: Ticket) -> Result<Self> {
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
        let random_config = RandomConfig::default();

        let topic_id = ticket.topic_id;
        let bootstrap = ticket.bootstrap.iter().cloned().collect();
        let (sender, receiver) = gossip.subscribe(topic_id, bootstrap).await?.split();

        let bc = BlockChain {router, downloader, blobs, tags, sender, game_gen, random_config, db_lock, new_block_signal};
        let bc2 = bc.clone();

        task::spawn(subscribe_loop(bc2, receiver));

        Ok(bc)
    }

    pub fn endpoint_id(&self) -> EndpointId {
        self.router.endpoint().id()
    }

    async fn print_state(&self) -> Result<()> {
        let head = self.get_head().await?;
        if !head.no_blocks() {
            let head_block = self.get_local_block(head.hash).await?;
            let converted_timestamp: DateTime<Local> = DateTime::from(head_block.timestamp);
            info!("\n\nCurrent height: {}\nCurrent hash: {:?}\nAt time: {:?}\nMined by: {:?}", head.height, head.hash, converted_timestamp, head_block.miner_name.clone());
        } else {
            info!("\n\nNEW CHAIN\n\n");
        }
        Ok(())
    }

    async fn get_head(&self) -> Result<BlockHead> {
        let ot = self.tags.get(String::from("head")).await?;
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
    }
    async fn set_head(&self, head: BlockHead) -> Result<()> {
        let h = self.blobs.add_bytes(head.encode()?).await?.hash;
        self.tags.set( String::from("head"), h).await?;
        Ok(())
    }

    // Block that might be foreign
    async fn get_foreign_block(&self, hash: Hash, peers: Vec<EndpointId>) -> Result<Block> {
        match self.get_local_block(hash).await {
            Ok(b) => Ok(b), // We have it locally
            Err(_e) => { // Try to get it from peers
                let s_peers = Shuffled::new(peers);

                let mut progress = self.downloader.download(hash, s_peers)
                    .stream().await?;

                while let Some(_event) = progress.next().await {}
                self.get_local_block(hash).await
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

    async fn new_block(&self, new_head_hash: Hash, peers: Vec<EndpointId>) -> Result<()> {
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
            if !self.evaluate_replay(&block).await? {
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

    async fn evaluate_replay(&self, block: &Block) -> Result<bool> {
        let mut game = self.game_gen.create_game()?;
        game.rng_init(block.calc_seed(), self.random_config)?;

        for p_pad in block.solution.clone().iter() {
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

    fn node(&self) -> Node {
        Node::new(self.endpoint_id())
    }

    async fn broadcast_block(&self, hash: Hash) -> Result<()> {
        let message = BlockMessage::NewBlockHead { node: self.node(), hash };
        let encoded = message.encode()?.to_vec();
        // sender.broadcast_neighbors(encoded).await?;
        match self.sender.broadcast(encoded.into()).await {
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
        let message = BlockMessage::RequestBlockHead{ node: self.node() };
        let encoded = message.encode()?.to_vec();
        match self.sender.broadcast(encoded.into()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::msg("Request head failed")),
        }
    }

    pub async fn start_mine(&self) {
        let mut _guard = self.db_lock.lock().await;
        {
            // If any new blocks are made while we're playing, then it will kill the recording session
            let mut new_block_signal = self.new_block_signal.lock().await;
            *new_block_signal = false;
        }
    }

    pub async fn submit_mine(&self, new_block: Block) -> Result<()> {
        let _guard = self.db_lock.lock().await;

        // Add block to blobs
        let new_hash = self.add_block_blob(new_block).await?;

        // You never have to download anything since you mined it locally, therefore no peers are needed

        let peers: Vec<EndpointId> = Vec::new();

        match self.new_block(new_hash, peers).await {
            Ok(_) => {
                self.broadcast_block(new_hash).await?;
                self.print_state().await?;
            },
            Err(_) => {info!("Submitting block failed")}
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
    pub async fn get_head_public(&self) -> Result<BlockHead> {
        let _guard = self.db_lock.lock().await;
        let head = self.get_head().await?;
        Ok(head)
    }

    pub async fn get_local_block_public(&self, hash: Hash) -> Result<Block> {
        let _guard = self.db_lock.lock().await;
        self.get_local_block(hash).await
    }
}

async fn subscribe_loop(
    bc: BlockChain, mut receiver: GossipReceiver
) {
    info!("SUBLOOP: Waiting to connect...");
    loop {
        if init_connection(&bc, &mut receiver).await.is_ok() {
            break;
        }
    }
    info!("SUBLOOP: Connected!");

    while let Some(e) = receiver.next().await {
        if e.is_err() {info!("SUB_LOOP API ERROR"); continue;}
        let event = e.unwrap();
        match process_event(&bc, &mut receiver, event).await {
            Ok(_) => {},
            Err(e) => info!("SUB_LOOP ERROR: {}", e.to_string()),
        }
    }
}

async fn init_connection(bc: &BlockChain, receiver: &mut GossipReceiver) -> Result<()> {
    receiver.joined().await?;
    {
        let _guard = bc.db_lock.lock().await;
        bc.request_head().await?;
        bc.broadcast_head().await?;
    }
    Ok(())
}

async fn process_event(bc: &BlockChain, receiver: &mut GossipReceiver, event: Event) -> Result<()> {
    let _guard = bc.db_lock.lock().await;
    info!("Event received!");

    if let Event::Received(msg) = event {
        // match msg.scope {
        //     Neighbors => {} // Only accept direct neighbour messages
        //     Swarm(_) => {return Err(Error::msg("Bad message scope"));}
        // }
        let message = BlockMessage::decode(&msg.content)?;
        match message {
            BlockMessage::NewBlockHead { hash, node: _ } => {
                // info!("Message: New Block Head");
                let peers: Vec<EndpointId> = receiver.neighbors().into_iter().collect();   
                bc.new_block(hash, peers).await?;
                bc.broadcast_head().await?;
                bc.print_state().await?;
            },
            BlockMessage::RequestBlockHead { node: _ } => {
                // info!("Message: Request Block Head");
                bc.broadcast_head().await?;
            }
        }
    }
    else if let Event::NeighborUp(key) = event {info!("Joined {}", key);}
    else if let Event::NeighborDown(key) = event {info!("Downed {}", key);}
    else if let Event::Lagged = event {info!("Lagged");};
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    endpoint_id: EndpointId,
    timestamp: DateTime<Utc>,
}

impl Node {
    pub fn new(endpoint_id: EndpointId) -> Self {
        Node {endpoint_id, timestamp: Utc::now() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum BlockMessage {
    NewBlockHead { node: Node, hash: Hash },
    RequestBlockHead { node: Node },
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