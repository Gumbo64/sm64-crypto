use std::{
    fmt, str::FromStr, u128
};

use bytes::Bytes;
use clap::Parser;
use futures_lite::StreamExt;
use iroh::{Endpoint, NodeAddr, PublicKey, RelayMode, Watcher };
use iroh_gossip::{
    api::{Event, GossipReceiver, GossipSender},
    net::Gossip,
    proto::TopicId,
};
use log::info;
use n0_future::task;
use n0_snafu::{Result, ResultExt, Error};
use serde::{Deserialize, Serialize};

use snafu::whatever;
use tokio::sync::Mutex;
use std::sync::Arc;

mod use_exes;

use use_exes::{ez_record_loop, ez_evaluate};
use sha2::{Sha256, Digest};


use iroh_blobs::{api::{ downloader::{Downloader, Shuffled}, tags::Tags }, store::mem::MemStore, store::fs::FsStore, BlobsProtocol, Hash };

use iroh::protocol::Router;
// use iroh_docs::{protocol::Docs};

#[derive(Parser, Debug)]
struct Args {
    /// secret key to derive our node id from.
    #[clap(long)]
    secret_key: Option<String>,
    #[clap(short, long)]
    name: Option<String>,
    #[clap(subcommand)]
    command: Command,
    /// Enable verbose output.
    #[clap(short, long, default_value_t = false)]
    record: bool,
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
        Ok(postcard::to_stdvec(&self).e().expect("Failed to encode block").into())
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
struct BlockChain {
    // head_hash: Hash,
    // head_height: u128,
    blobs: BlobsProtocol,
    tags: Tags,
    downloader: Downloader,
    sender: GossipSender,
    peers_lock: Arc<Mutex<Vec<PublicKey>>>,
    // receiver: GossipReceiver,
}

impl BlockChain {
    pub async fn new(endpoint: Endpoint, ticket: Option<Ticket>) -> Option<(BlockChain, GossipReceiver)> {
        // Create protocols

        let is_hosting = ticket.is_none();


        let store_path = format!("my_blockchain_{}", is_hosting);
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

        // Absorb ticket
        let mut topic = TopicId::from_bytes(rand::random());
        let mut peers = vec![];
        if let Some(t) = ticket {
           topic = t.topic;
           peers = t.peers;
        }

        // Join peers
        let peer_ids = peers.iter().map(|p| p.node_id).collect();
        for peer in peers.clone().into_iter() {
            endpoint.add_node_addr(peer).ok();
        }

        // Join/create gossip chat
        let (sender, receiver) = gossip.subscribe(topic, peer_ids).await.ok()?.split();

        // Print new ticket including us
        let ticket_with_me: Ticket = {
            let me = endpoint.node_addr().initialized().await;
            let peers = peers.iter().cloned().chain([me]).collect();
            Ticket { topic, peers }
        };
        println!("> ticket to join us: {ticket_with_me}");

        let peers_lock = Arc::new(Mutex::new(receiver.neighbors().collect()));

        Some((BlockChain { blobs: blobs.clone(), tags: tags.clone(), downloader, sender, peers_lock }, receiver))
    }

    async fn get_head(&self) -> Result<BlockHead> {
        let ott = self.tags.get(String::from("head")).await;
        
        match ott {
            Ok(ot) => {
                let t = ot.e()?;
                let bytes = self.blobs.get_bytes(t.hash).await?;
                return BlockHead::decode(&bytes);
            }
            Err(e) => {
                println!("{}",e);
                self.set_head(BlockHead::default()).await?;
                return Ok(BlockHead::default());
            }
        }

    }
    async fn set_head(&self, head: BlockHead) -> Result<()> {
        let h= self.blobs.add_bytes( postcard::to_stdvec(&head).e()?).await?.hash;
        self.tags.set( String::from("head"), h).await?;
        Ok(())
    }

    async fn get_block(&self, hash: Hash) -> Result<Block> {
        let block_bytes: Bytes = match self.blobs.get_bytes(hash).await {
            Ok(b) => b, // We have it locally
            Err(_e) => { // Try to get it from peers
                let peers = Shuffled::new(self.peers_lock.lock().await.to_vec());
                let mut progress = self.downloader.download(hash, peers)
                    .stream().await.e()?;
                while let Some(event) = progress.next().await {
                    info!("Progress: {:?}", event);
                }
                self.blobs.get_bytes(hash).await?
            }
        };
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
        let block = self.get_block(hash).await.e()?;
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

    async fn new_block(&self, new_head_hash: Hash) -> Result<()> {

        let new_head = self.get_block(new_head_hash).await.e()?;

        // check that the new head is even worth it, it should have a higher head
        let head = self.get_head().await.expect("Failed to get head");
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
            block = self.get_block(cur_hash).await.e()?;

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
            println!("Confirming {i}");
            self.confirm_block(i).await?;
        }

        let new_blockhead = BlockHead {hash: new_head_hash, height: new_head.block_height };
        self.set_head(new_blockhead).await.e()?;
        self.clear_temp_blocks().await?;


        println!("L");
        self.broadcast_block(new_head_hash).await.e()?;

        Ok(())
    }

    async fn broadcast_block(&self, hash: Hash) -> Result<()> {
        let message = Message::NewBlockHead { hash };
        let encoded = message.encode().e()?;
        self.sender.broadcast_neighbors(encoded).await.e()?;
        Ok(())
    }

    async fn broadcast_head(&self) -> Result<()> {
        let head = self.get_head().await.e()?;
        if head.no_blocks(){
            return Ok(());
        }
        self.broadcast_block(head.hash).await
    }
}
async fn sync_blockchain(blockchain: BlockChain, mut receiver: GossipReceiver) -> Result<()> {
    while let Some(event) = receiver.try_next().await? {
        if let Event::Received(msg) = event {
            let message = Message::decode(&msg.content)?;
            match message {
                Message::NewBlockHead { hash } => {
                    let _ = blockchain.new_block(hash).await;
                }
                Message::RequestBlockHead {} => {
                    let _ = blockchain.broadcast_head().await;
                    // blocks_map.insert(from, name.clone());
                    // println!("{name}: {text}");
                }
                // Message::RequestBlock { link } => {
                //     // blocks_map.insert(from, name.clone());
                //     // println!("> {} is now known as {}", from.fmt_short(), name);
                // }
                // Message::ResponseBlock { block } => {
                //     // let name = names
                //         // .get(&from)
                //         // .map_or_else(|| from.fmt_short(), String::to_string);
                //     // println!("{name}: {text}");
                // }
            }
            let mut peers = blockchain.peers_lock.lock().await;
            *peers = receiver.neighbors().collect();
        }
    }
    Ok(())
}


#[derive(Debug, Serialize, Deserialize, Clone)]
struct Block {
    prev_hash: Hash,
    block_height: u128,
    solution_bytes: Vec<u8>,
}

impl Block {
    pub fn new(block_head: BlockHead) -> Result<Self> {
        // Genesis block, these are both true. Normal block, neither are true
        // if (block_head.hash == Hash::EMPTY) != (block_head.height == u128::MAX) {
        //     whatever!("Head doesn't make sense")
        // }
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
        Ok(postcard::to_stdvec(&self).e().expect("Failed to encode block").into())
    }

    pub fn decode(bytes: &[u8]) -> Result<Block> {
        Ok(postcard::from_bytes(bytes).e()?)
    }

}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let endpoint = Endpoint::builder().relay_mode(RelayMode::Default).bind().await?;

    let (blockchain, receiver) = match &args.command {
        Command::Open{} => { // No ticket
            BlockChain::new(endpoint, None).await.expect("Blockchain creation failed")
        }
        Command::Join { ticket } => { // Ticket provided
            let t = Ticket::from_str(ticket).expect("Ticket is invalid");
            BlockChain::new(endpoint, Some(t)).await.expect("Blockchain creation failed")
        }
    };


    task::spawn(sync_blockchain(blockchain.clone(), receiver));

    loop {
        if args.record {
            let res: Result<()> = {
                println!("AAAA");
                let head = blockchain.get_head().await.e()?;

                // Mine a block
                let new_block = Block::new(head).e()?;
                println!("BBBB");

                let new_hash = blockchain.add_block_blob(new_block).await.e()?;
                println!("CCCC");

                blockchain.new_block(new_hash).await
            };
            match res {
                Ok(_) => continue,
                Err(s) => println!("{:?}", s)
            }
            println!("EEEE");

        
        }
        // let _ = blockchain.sync().await;
        // blockchain.clear_temp_blocks().await;
    }

}



#[derive(Debug, Serialize, Deserialize)]
enum Message {
    NewBlockHead { hash: Hash },
    RequestBlockHead { },
}

impl Message {
    pub fn decode(bytes: &[u8]) -> Result<Message> {
        Ok(postcard::from_bytes(bytes).e()?)
    }

    fn encode(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(&self).e().expect("Failed to encode message").into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Ticket {
    topic: TopicId,
    peers: Vec<NodeAddr>,
}
impl Ticket {
    /// Deserializes from bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        postcard::from_bytes(bytes).e()
    }
    /// Serializes to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        postcard::to_stdvec(self).expect("postcard::to_stdvec is infallible")
    }
}

/// Serializes to base32.
impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text = data_encoding::BASE32_NOPAD.encode(&self.to_bytes()[..]);
        text.make_ascii_lowercase();
        write!(f, "{text}")
    }
}

/// Deserializes from base32.
impl FromStr for Ticket {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_NOPAD
            .decode(s.to_ascii_uppercase().as_bytes())
            .e()?;
        Self::from_bytes(&bytes)
    }
}

