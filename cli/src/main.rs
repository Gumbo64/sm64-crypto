use clap::Parser;

use n0_snafu::Result;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tokio::sync::Mutex;

mod use_exes;
use use_exes::{record_loop, ez_evaluate};
use sm64_crypto_shared::{BlockChain, Block, DEFAULT_CONFIG, Config};

const CONFIG: Config = DEFAULT_CONFIG;

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
    // db_lock: Arc<Mutex<()>>, new_block_signal: Arc<Mutex<bool>>, block_eval_request: Arc<Mutex<Option<(Block, Option<bool>)>>>,

fn parse_miner_name(s: String) -> [u8; CONFIG.max_name_length] {
    let mut miner_name: [u8; CONFIG.max_name_length] = [0; CONFIG.max_name_length];
    let vec = s;
    assert!(vec.len() <= CONFIG.max_name_length);
    let copy_length = vec.len().min(CONFIG.max_name_length);
    miner_name[..copy_length].copy_from_slice(&vec.as_bytes()[..copy_length]);
    miner_name
}

async fn mine_attempt(bc: &BlockChain, miner_name:[u8; CONFIG.max_name_length] ) -> Result<()> {
    let (mut block, kill_signal) = bc.start_mine(miner_name).await?;
    let seed = block.calc_seed();
    block.seal(record_loop(seed, kill_signal, CONFIG)?);
    bc.submit_mine(block).await?;
    Ok(())
}

async fn process_eval_request(block_eval_request: Arc<Mutex<Option<(Block, Option<bool>)>>>) -> Option<()> {
    let mut e_request = block_eval_request.lock().await;
    let (block, _) = (*e_request)?;
    let seed = block.calc_seed();
    let solution_bytes = block.get_solution();
    let success = ez_evaluate(seed, &solution_bytes, true, CONFIG);
    *e_request = Some((block, Some(success)));
    Some(())
}

async fn show_head(bc: &BlockChain) -> Result<()> {
    let block_head = bc.get_head_public().await?;
    let block = bc.get_local_block_public(block_head.hash).await?;
    ez_evaluate(block.calc_seed(), &block.get_solution(), false, CONFIG);
    Ok(())
}

async fn is_new_block(sig: Arc<Mutex<bool>>) -> bool {
    // Check if there is a new block
    let mut nb_p = sig.lock().await;
    if *nb_p {
        *nb_p = false;
        return true;
    }
    false
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");



    println!("Starting SM64-Crypto");
    let miner_name = parse_miner_name(args.miner_name);


    let (bc, block_eval_request) = BlockChain::new(args.nowait).await.expect("");

    tokio::spawn(async move {
        loop {
            if !process_eval_request(block_eval_request.clone()).await.is_none() {
                println!("Processed a block");
            }
            // Sleep for a specified duration before the next iteration
            time::sleep(Duration::from_millis(200)).await;

        }
    });

    if args.mine {
        while running.load(Ordering::SeqCst) {
            match mine_attempt(&bc, miner_name).await {
                Ok(_) => {

                },
                Err(_e) => {
                    println!("Resetting the game so we're mining from the newest block");
                    if args.showblocks {
                        let _ = show_head(&bc).await;
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

            let new_block_signal = bc.get_new_block_signal();
            if args.showblocks && is_new_block(new_block_signal).await {
                let _ = show_head(&bc).await;
            }
            match show_head(&bc).await {
                Ok(_e) => {}
                Err(e) => {
                    println!("{}\n",e);
                }
            }
        }
    }

    println!("Ending the program");
}
