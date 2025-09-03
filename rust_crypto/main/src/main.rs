use core::hash;
use std::{
    clone, collections::HashMap, fmt, hash::Hasher, net::{Ipv4Addr, SocketAddrV4}, result, str::FromStr, u128
};

use anyhow::Context;
use bytes::Bytes;
use clap::Parser;
use ed25519_dalek::Signature;
use futures_lite::StreamExt;
use iroh::{Endpoint, NodeAddr, PublicKey, RelayMode, RelayUrl, SecretKey, Watcher};
use iroh_gossip::{
    api::{Event, GossipReceiver},
    net::{Gossip},
    proto::TopicId,
};
use n0_future::task;
use n0_snafu::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use snafu::whatever;

mod use_exes;
use rand::{rand_core::block, Rng};
use use_exes::{ez_record_loop, ez_evaluate};
use sha2::{Sha256, Digest};

use std::sync::{Arc, Mutex};
use tracing::info;

use iroh_blobs::{api::{blobs::Blobs, downloader::Downloader, Store}, get::request::GetBlobItem, store::mem::MemStore, ticket::BlobTicket, BlobFormat, BlobsProtocol, Hash};
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

#[derive(Clone)]
struct BlockHead {
    hash: Hash,
    block_height: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    // parse the cli command
    let (topic, peers) = match &args.command {
        Command::Open{} => {
            let topic = TopicId::from_bytes(rand::random());
            // println!("> opening chat room for topic {topic}");
            (topic, vec![])
        }
        Command::Join { ticket } => {
            let Ticket { topic, peers } = Ticket::from_str(ticket)?;
            // println!("> joining chat room for topic {topic}");
            (topic, peers)
        }
    };


    // configure our relay map
    let relay_mode = RelayMode::Default;
    // println!("> using relay servers: {}", fmt_relay_mode(&relay_mode));

    // build our magic endpoint
    let endpoint = Endpoint::builder()
        .relay_mode(relay_mode)
        // .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, args.bind_port))
        .bind()
        .await?;
    // println!("> our node id: {}", endpoint.node_id());


    // create a protocol handler using an in-memory blob store.
    let store = MemStore::new();

    let blobs = BlobsProtocol::new(&store, endpoint.clone(), None);
    // create the gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone());

    // let docs = Docs::memory()
    //     .spawn(endpoint.clone(), (*blobs).clone(), gossip.clone());

    // print a ticket that includes our own node id and endpoint addresses
    let ticket = {
        let me = endpoint.node_addr().initialized().await;
        let peers = peers.iter().cloned().chain([me]).collect();
        Ticket { topic, peers }
    };
    println!("> ticket to join us: {ticket}");

    // setup router
    let router = Router::builder(endpoint.clone())
        .accept(iroh_blobs::ALPN, blobs.clone())
        .accept(iroh_gossip::ALPN, gossip.clone())
        // .accept(iroh_docs::ALPN, docs.clone())
        .spawn();

    // join the gossip topic by connecting to known peers, if any
    let peer_ids = peers.iter().map(|p| p.node_id).collect();
    if peers.is_empty() {
        // println!("> waiting for peers to join us...");
    } else {
        // println!("> trying to connect to {} peers...", peers.len());
        // add the peer addrs from the ticket to our endpoint's addressbook so that they can be dialed
        for peer in peers.into_iter() {
            endpoint.add_node_addr(peer)?;
        }
    };
    let (sender, receiver) = gossip.subscribe(topic, peer_ids).await?.split();
    // println!("> connected!");



    // subscribe and print loop
    let mut local_head = BlockHead { hash: Hash::EMPTY, block_height: u128::MAX };
    let head_lock: Arc<Mutex<BlockHead>> = Arc::new(Mutex::new(local_head.clone()));

    let downloader = store.downloader(&endpoint);
    task::spawn(subscribe_loop(downloader, blobs.clone(), receiver, Arc::clone(&head_lock)));



    while args.record {
    // loop {
        if let Ok(bh) = head_lock.lock() {
            local_head = bh.clone();
        } else {
            continue;
        }

        let block_opt = Block::new(&local_head);
        if let Some(block) = block_opt {
            let hash = blobs.add_bytes(block.encode()?).await?.hash;
            let node_addr = endpoint.node_addr().initialized().await;
            let ticket = BlobTicket::new(node_addr, hash, BlobFormat::Raw);
            
            let message = Message::NewBlockHead { link: ticket };
            
            let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
            sender.broadcast(encoded_message).await?;
            println!("> sent: {}", local_head.block_height);

            loop {
                if let Ok(mut bh) = head_lock.lock() {
                    bh.hash = hash;
                    bh.block_height = block.block_height;
                    println!("{} {}", bh.block_height, bh.hash);
                    break;
                }
            }

        } else {
            // println!("Some fail");
        }

    }

    
    let _ = tokio::signal::ctrl_c().await;


    // shutdown
    router.shutdown().await.e()?;

    Ok(())
}

// fn get_block()

async fn subscribe_loop(downloader: Downloader, blobs: BlobsProtocol, mut receiver: GossipReceiver, mut head_lock: Arc<Mutex<BlockHead>>) -> Result<()> {
    // init a peerid -> name hashmap
    // let mut names: HashMap<_, String> = HashMap::new();
    while let Some(event) = receiver.try_next().await? {
        if let Event::Received(msg) = event {
            let (_from, message) = SignedMessage::verify_and_decode(&msg.content)?;
            match message {
                Message::NewBlockHead { link } => {
                    let progress = downloader
                            .download(link.hash(), Some(link.node_addr().node_id))
                            .stream()
                            .await;
                    match progress {
                        Ok(mut stream) => {
                            // Handle the successful case
                            while let Some(_event) = stream.next().await {
                                // info!("Progress: {:?}", event);
                            }

                            let block_bytes: Bytes = blobs.get_bytes(link.hash()).await?;
                            let block: Block = postcard::from_bytes(block_bytes.as_ref()).e()?;
                            
                            if block.evaluate(&blobs).await {
                                loop {
                                    if let Ok(mut bh) = head_lock.lock() {
                                        if block.block_height > bh.block_height || bh.block_height == u128::MAX {
                                            bh.hash = link.hash();
                                            bh.block_height = block.block_height;   
                                            println!("{} {}", bh.block_height, bh.hash);
                                        }
                                        break;
                                    }
                                }
                                println!("> The block evaluates to true");
                            } else {
                                println!("> The block evaluates to false");
                            }

                            
                        }
                        Err(e) => {
                            // Handle the error case
                            continue;
                        }
                    }

                }
                Message::RequestBlockHead { link } => {
                    // blocks_map.insert(from, name.clone());
                    // println!("{name}: {text}");
                }
                Message::RequestBlock { link } => {
                    // blocks_map.insert(from, name.clone());
                    // println!("> {} is now known as {}", from.fmt_short(), name);
                }
                Message::ResponseBlock { block } => {
                    // let name = names
                        // .get(&from)
                        // .map_or_else(|| from.fmt_short(), String::to_string);
                    // println!("{name}: {text}");
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct SignedMessage {
    from: PublicKey,
    data: Bytes,
    signature: Signature,
}

impl SignedMessage {
    pub fn verify_and_decode(bytes: &[u8]) -> Result<(PublicKey, Message)> {
        let signed_message: Self = postcard::from_bytes(bytes).e()?;
        let key: PublicKey = signed_message.from;
        key.verify(&signed_message.data, &signed_message.signature)
            .e()?;
        let message: Message = postcard::from_bytes(&signed_message.data).e()?;
        Ok((signed_message.from, message))
    }

    pub fn sign_and_encode(secret_key: &SecretKey, message: &Message) -> Result<Bytes> {
        let data: Bytes = postcard::to_stdvec(&message).e()?.into();
        let signature = secret_key.sign(&data);
        let from: PublicKey = secret_key.public();
        let signed_message = Self {
            from,
            data,
            signature,
        };
        let encoded = postcard::to_stdvec(&signed_message).e()?;
        Ok(encoded.into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    NewBlockHead { link: BlobTicket },
    RequestBlockHead { link: BlobTicket },
    RequestBlock { link: BlobTicket },
    ResponseBlock { block: Block },
}

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    prev_hash: Hash,
    block_height: u128,
    solution_bytes: Vec<u8>,
}

impl Block {
    pub fn new(block_head: &BlockHead) -> Option<Self> {
        // Genesis block, these are both true. Normal block, neither are true
        if (block_head.hash == Hash::EMPTY) != (block_head.block_height == u128::MAX) {
            return None;
        }
        let prev_hash = block_head.hash;
        let block_height = block_head.block_height.wrapping_add(1);

        let seed = Block {prev_hash, block_height, solution_bytes: Vec::new() }.calc_seed();
        let solution_bytes = ez_record_loop(&seed);
        if solution_bytes.len() == 0 {
            return None
        }

        Some(Block { prev_hash, block_height, solution_bytes })
    }
    async fn evaluate(&self, blobs: &BlobsProtocol) -> bool {
        if self.block_height != 0 {
            let prev_block_bytes: Bytes = match blobs.get_bytes(self.prev_hash).await {
                Ok(bytes) => bytes,
                Err(_) => return false
            };
            let prev_block: Block = match postcard::from_bytes(prev_block_bytes.as_ref()).e() {
                Ok(block) => block,
                Err(_) => return false
            };
            if prev_block.block_height + 1 != self.block_height {
                return false;
            }
        }

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
        let encoded = postcard::to_stdvec(&self).e()?;
        Ok(encoded.into())
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
    type Err = n0_snafu::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_NOPAD
            .decode(s.to_ascii_uppercase().as_bytes())
            .e()?;
        Self::from_bytes(&bytes)
    }
}

