use clap::Parser;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use tokio::time;

use anyhow::Result;
use tracing::info;

use sm64_blockchain::BlockChainClient;

#[derive(Parser, Debug)]
struct Args {
    // /// Enable mining
    // #[clap(short, long, default_value_t = false)]
    // mine: bool,
    // // When we receive successful blocks, show them to the user
    // #[clap(short, long, default_value_t = false)]
    // showblocks: bool,
    #[clap(short, long, default_value_t = String::from("Gumbo64"))]
    name: String,
    #[clap(short, long, default_value_t = String::from(""))]
    ticket: String,
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

    info!("Starting SM64-Crypto");

    let rom_bytes = include_bytes!("../../baserom.us.z64").to_vec();
    
    let ticket_opt = match args.ticket.len() == 0 {
        true => None,
        false => Some(args.ticket),
    };

    let bc_client = BlockChainClient::new(rom_bytes, args.name, ticket_opt).await.expect("Failed to create blockchain client");

    info!("Join us at:\n{}\n", bc_client.get_ticket());

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

    info!("Ending the program");
    Ok(())
}
