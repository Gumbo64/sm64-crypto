
use clap::Parser;

use n0_snafu::{Result, };

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;


mod blockchain;
use blockchain::BlockChain;


#[derive(Parser, Debug)]
struct Args {
    /// Enable mining
    #[clap(short, long, default_value_t = false)]
    mine: bool,
    /// Wait for a connection before starting
    #[clap(short, long, default_value_t = false)]
    nowait: bool,
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

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let bc = BlockChain::new(args.nowait).await?;

    while running.load(Ordering::SeqCst) {
        if args.mine {
            bc.mine().await?;
        }
    }

    println!("Ending the program");
    Ok(())
}
