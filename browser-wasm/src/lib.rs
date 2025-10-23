
use anyhow::{Result, Error};

use n0_future::TryFutureExt;
use tracing::level_filters::LevelFilter;
use tracing_subscriber_wasm::MakeConsoleWriter;
use wasm_bindgen::{JsError, prelude::wasm_bindgen};
use n0_future::{
    StreamExt,
    boxed::BoxStream,
    task::{self, AbortOnDropHandle},
    time::{Duration, SystemTime},
};

use std::{
    sync::{Arc, Mutex}
};
use iroh_blobs::Hash;


use sm64_crypto_shared::{BlockChainClient};


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


/// Blockchain node using Iroh
#[wasm_bindgen]
pub struct BlockChainClientWeb {
    client: BlockChainClient
}

#[wasm_bindgen]
impl BlockChainClientWeb {
    pub async fn new(miner_name: String, nowait: bool) -> Result<Self, JsError> {
        let client = BlockChainClient::new(miner_name, nowait)
            .await
            .map_err(to_js_err)?;
        Ok(Self {client})
    }

    pub async fn start_mine(&mut self) -> Result<u32, JsError> {
        self.client.start_mine().await.map_err(to_js_err)
    }

    pub async fn submit_mine(&mut self, seed: u32, solution: Vec<u8>) -> Result<(), JsError> {
        self.client.submit_mine(seed, solution).await.map_err(to_js_err)
    }

    pub async fn get_eval_request(&self) -> Result<Vec<u8>, JsError> {
        let (seed, solution) = self.client.get_eval_request().await.map_err(to_js_err)?;

        let mut result = Vec::new();
        result.extend(seed.to_le_bytes());
        result.extend(solution);
        Ok(result)
    }

    pub async fn respond_eval_request(&self, seed: u32, valid: bool) -> Result<(), JsError> {
        self.client.respond_eval_request(seed, valid).await.map_err(to_js_err)
    }

    pub async fn get_head_bytes(&self) -> Result<Vec<u8>, JsError> {
        let block = self.client.get_head().await.map_err(to_js_err)?;
        Ok(block.get_solution())
    }

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