
use anyhow::{Result, Error};

use tracing::level_filters::LevelFilter;
use tracing_subscriber_wasm::MakeConsoleWriter;
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

use iroh_blobs::Hash;


use sm64_blockchain::{BlockChainClient, GamePad};

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .with_writer(
            // To avoide trace events in the browser from showing their JS backtrace
            MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG),
        )
        // If we don't do this in the browser, we get a runtime error.
        .without_time()
        .with_ansi(false)
        .init();

    tracing::info!("(testing logging) Logging setup");
}

#[wasm_bindgen]
pub struct GamePadWeb {
    button: u16,
    stick_x: i8,
    stick_y: i8,
}

/// Blockchain node using Iroh
#[wasm_bindgen]
pub struct BlockChainClientWeb {
    client: BlockChainClient
}

#[wasm_bindgen]
impl BlockChainClientWeb {
    pub async fn new(miner_name: String, ticket_str: String) -> Result<Self, JsError> {
        let client = BlockChainClient::new(miner_name, ticket_str)
            .await
            .map_err(to_js_err)?;
        Ok(Self {client})
    }

    pub fn get_ticket(&self) -> Result<String, JsError> {
        Ok(self.client.get_ticket())
    }

    pub async fn start_mine(&mut self) -> Result<u32, JsError> {
        self.client.start_mine().await.map_err(to_js_err)
    }

    pub async fn submit_mine(&mut self, seed: u32, solution: Vec<GamePadWeb>) -> Result<(), JsError> {
        // let mut solution_pads: Vec<GamePad> = Vec::new();
        // for chunk in solution.chunks(4) {
        //     if chunk.len() == 4 {
        //         // Only create GamePad if there's a complete chunk of 4 bytes
        //         let pad = GamePad::from_bytes(chunk);
        //         solution_pads.push(pad);
        //     } else {
        //         eprintln!("Warning: Incomplete chunk ignored: {:?}", chunk);
        //     }
        // }

        let mut solution_pads: Vec<GamePad> = Vec::new();
        for webpad in solution {
            let pad = GamePad::new(webpad.button, webpad.stick_x, webpad.stick_y);
            solution_pads.push(pad);
        }

        self.client.submit_mine(seed, solution_pads).await.map_err(to_js_err)
    }
    // pub async fn get_head_bytes(&self) -> Result<Vec<u8>, JsError> {
    //     let block = self.client.get_head().await.map_err(to_js_err)?;
    //     Ok(block.get_solution())
    // }

    pub async fn has_new_block(&self) -> bool {
        self.client.has_new_block().await
    }

    pub async fn get_block_head_json(&self) -> Result<String, JsError> {
        let block = self.client.get_head().await.map_err(to_js_err)?;
        Ok(serde_json::to_string_pretty(&block)?)
    }

    pub async fn get_block_json(&self, hash: String) -> Result<String, JsError> {
        let hash_bytes = hash.as_bytes();
        let l1 = hash_bytes.len();
        let l2 = Hash::EMPTY.as_bytes().len();
        if l1 != l2 {
            println!("hash lengths: {} {}\n", l1, l2);
            return Err(Error::msg("Provided hash is of the wrong length, might be whitespace")).map_err(to_js_err);
        }
        let mut array: [u8; 32] = [0u8; 32];
        array[..hash_bytes.len()].copy_from_slice(hash_bytes);

        let block = self.client.get_block(Hash::from_bytes(array)).await.map_err(to_js_err)?;
        Ok(serde_json::to_string_pretty(&block)?)
    } 
}

fn to_js_err(err: impl Into<anyhow::Error>) -> JsError {
    let err: anyhow::Error = err.into();
    JsError::new(&err.to_string())
}