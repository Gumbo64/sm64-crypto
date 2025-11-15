use clap::Parser;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use tokio::time;
// use tokio::sync::Mutex;

// use n0_future::{
//     StreamExt,
//     boxed::BoxStream,
//     task::{self, AbortOnDropHandle},
//     time::{Duration, SystemTime},
// };

use anyhow::Result;

// mod use_exes;
// use use_exes::{record_loop, ez_evaluate};
use sm64_blockchain::BlockChainClient;

#[derive(Parser, Debug)]
struct Args {
    /// Enable mining
    #[clap(short, long, default_value_t = false)]
    mine: bool,
    // When we receive successful blocks, show them to the user
    #[clap(short, long, default_value_t = false)]
    showblocks: bool,
    #[clap(short, long, default_value_t = String::from("Gumbo64"))]
    name: String,
    #[clap(short, long, default_value_t = String::from(""))]
    ticket: String,
}

// async fn mine_attempt(bc_client: &mut BlockChainClient) -> Result<()> {
//     let seed = bc_client.start_mine().await?;
//     let kill_signal = Arc::new(Mutex::new(false));
//     let kill_signal_clone = Arc::clone(&kill_signal);

//     // Spawn a thread to monitor for new blocks
//     let _kill_detect_task = AbortOnDropHandle::new(task::spawn({
//         let b = bc_client.clone();
//         async move {
//             loop {
//                 if b.has_new_block().await {
//                     let mut signal = kill_signal_clone.lock().await;
//                     *signal = true; // Set kill_signal to true if a new block is found
//                     break; // Exit the loop if a new block is detected
//                 }
//             }
//         }
//     }));
//     let solution = record_loop(seed, kill_signal.clone(), DEFAULT_CONFIG)?;

//     // Submit the mining result
//     bc_client.submit_mine(seed, solution).await?;

//     Ok(())
// }

// async fn show_head(bc_client: &BlockChainClient) -> Result<()> {
//     let block = bc_client.get_head().await?;
//     ez_evaluate(block.calc_seed(), block.get_solution(), false, DEFAULT_CONFIG);
//     Ok(())
// }


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

    let bc_client = BlockChainClient::new(args.name, args.ticket).await.expect("Failed to create blockchain client");

    println!("Join us:\n {}\n", bc_client.get_ticket());

    loop {
        time::sleep(time::Duration::from_secs(1)).await;
        // Check running state if needed
        if !running.load(Ordering::SeqCst) {
            break; // Exit the loop if running is false
        }

        // if args.showblocks && bc_client.has_new_block().await {
        //     let _ = show_head(&bc_client).await;
        // }
    }

    println!("Ending the program");
    Ok(())
}
