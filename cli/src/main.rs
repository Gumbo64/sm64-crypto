
use clap::Parser;

use n0_snafu::Result;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time;

use sm64_crypto_shared::{BlockChain, MAX_NAME_LENGTH};

#[derive(Parser, Debug)]
struct Args {
    /// Enable mining
    #[clap(short, long, default_value_t = false)]
    mine: bool,
    /// Wait for a connection before starting
    #[clap(short, long, default_value_t = false)]
    nowait: bool,
    // When we receive successful blocks, show them to the user
    #[clap(short, long, default_value_t = false)]
    showblocks: bool,
    #[clap(short, long, default_value_t = String::from("Gumbo64"))]
    miner_name: String,
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


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    println!("Starting SM64-Crypto");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let bc = BlockChain::new(args.nowait, args.showblocks).await?;

    let mut miner_name: [u8; MAX_NAME_LENGTH] = [0; MAX_NAME_LENGTH];
    let vec = args.miner_name;
    assert!(vec.len() <= MAX_NAME_LENGTH);
    let copy_length = vec.len().min(MAX_NAME_LENGTH);
    miner_name[..copy_length].copy_from_slice(&vec.as_bytes()[..copy_length]);


    if args.mine {
        while running.load(Ordering::SeqCst) {
            bc.mine(miner_name).await;
        }
    } else {
        loop {
            time::sleep(time::Duration::from_secs(1)).await;
            // Check running state if needed
            if !running.load(Ordering::SeqCst) {
                break; // Exit the loop if running is false
            }
        }
    }

    

    println!("Ending the program");
    Ok(())
}
