mod use_exes;
use rand::Rng;
use use_exes::{ez_record_loop};

// fn main() {
//     let seed = "yeah cuz";

//     let solution_bytes = ez_record_loop(seed);

//     println!("{}\n", solution_bytes.len());
// }


use anyhow::Result;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_gossip::{net::Gossip, proto::TopicId};
use iroh_blobs::{store::mem::MemStore, BlobsProtocol};

#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = Endpoint::builder().discovery_n0().bind().await?;

    // create a protocol handler using an in-memory blob store.
    let store = MemStore::new();
    let blobs = BlobsProtocol::new(&store, endpoint.clone(), None);

    println!("> our node id: {}", endpoint.node_id());
    let gossip = Gossip::builder().spawn(endpoint.clone());

    let router = Router::builder(endpoint.clone())
        .accept(iroh_blobs::ALPN, blobs.clone())
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();


    let tag_string = format!("RandomNumber_{}", rand::rng().random_range(0..100));

    let tag = blobs.add_slice(tag_string).await?;
    println!("We are now serving {}", blobs.ticket(tag).await?);


    // Create a new topic.
    let id = TopicId::from_bytes(rand::random());
    println!("{}",id);
    let node_ids = vec![];

    // Subscribe to the topic.
    // Since the `node_ids` list is empty, we will
    // subscribe to the topic, but not attempt to
    // connect to any other nodes.
    let topic = gossip.subscribe(id, node_ids);

    // `split` splits the topic into the `GossipSender`
    // and `GossipReceiver` portions
    let (sender, _receiver) = topic.await?.split();

    // Broadcast a message to the topic.
    // Since no one else is a part of this topic,
    // this message is currently going out to no one.
    sender.broadcast("sup".into()).await?;

    // Wait for exit
    tokio::signal::ctrl_c().await?;

    router.shutdown().await?;

    Ok(())
}


