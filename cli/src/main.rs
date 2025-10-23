use clap::Parser;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use tokio::time;
use tokio::sync::Mutex;

use n0_future::{
    StreamExt,
    boxed::BoxStream,
    task::{self, AbortOnDropHandle},
    time::{Duration, SystemTime},
};

use anyhow::Result;

mod use_exes;
use use_exes::{record_loop, ez_evaluate};
use sm64_crypto_shared::{BlockChainClient, DEFAULT_CONFIG};

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



async fn mine_attempt(bc_client: &mut BlockChainClient) -> Result<()> {
    let seed = bc_client.start_mine().await?;
    let kill_signal = Arc::new(Mutex::new(false));
    let kill_signal_clone = Arc::clone(&kill_signal);

    // Spawn a thread to monitor for new blocks
    let _kill_detect_task = AbortOnDropHandle::new(task::spawn({
        let b = bc_client.clone();
        async move {
            loop {
                if b.has_new_block().await {
                    let mut signal = kill_signal_clone.lock().await;
                    *signal = true; // Set kill_signal to true if a new block is found
                    break; // Exit the loop if a new block is detected
                }
            }
        }
    }));
    let solution = record_loop(seed, kill_signal.clone(), DEFAULT_CONFIG)?;

    // Submit the mining result
    bc_client.submit_mine(seed, solution).await?;

    Ok(())
}

async fn process_eval_request(bc_client: &BlockChainClient) -> Result<()> {
    let (seed, solution) = bc_client.get_eval_request().await?;
    let valid = ez_evaluate(seed, solution, true, DEFAULT_CONFIG);
    bc_client.respond_eval_request(seed, valid).await
}

async fn show_head(bc_client: &BlockChainClient) -> Result<()> {
    let block = bc_client.get_head().await?;
    ez_evaluate(block.calc_seed(), block.get_solution(), false, DEFAULT_CONFIG);
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()>{
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");



    println!("Starting SM64-Crypto");

    let mut bc_client = BlockChainClient::new(args.miner_name, args.nowait).await.expect("Failed to create blockchain client");

    tokio::spawn({
        let b = bc_client.clone();
        async move {
            loop {
                if process_eval_request(&b).await.is_ok() {
                    println!("Processed a block");
                }
                // Sleep for a specified duration before the next iteration
                time::sleep(Duration::from_millis(200)).await;
            }
        }
    });

    if args.mine {
        while running.load(Ordering::SeqCst) {
            match mine_attempt(&mut bc_client).await {
                Ok(_) => {

                },
                Err(_e) => {
                    println!("Resetting the game so we're mining from the newest block");
                    if args.showblocks {
                        let _ = show_head(&bc_client).await;
                    }
                }
            }
        }
    } else {
        loop {
            time::sleep(time::Duration::from_secs(1)).await;
            // Check running state if needed
            if !running.load(Ordering::SeqCst) {
                break; // Exit the loop if running is false
            }

            if args.showblocks && bc_client.has_new_block().await {
                let _ = show_head(&bc_client).await;
            }
        }
    }

    println!("Ending the program");
    Ok(())
}
