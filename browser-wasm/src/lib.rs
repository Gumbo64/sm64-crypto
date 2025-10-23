
use anyhow::{Result, Error};

use n0_future::TryFutureExt;
use tracing::level_filters::LevelFilter;
use tracing_subscriber_wasm::MakeConsoleWriter;
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

use std::{sync::Arc};
use tokio::sync::Mutex;

// use sm64_crypto_shared::{Block, BlockChain};


// #[wasm_bindgen(start)]
// fn start() {
//     console_error_panic_hook::set_once();

//     tracing_subscriber::fmt()
//         .with_max_level(LevelFilter::DEBUG)
//         .with_writer(
//             // To avoide trace events in the browser from showing their JS backtrace
//             MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG),
//         )
//         // If we don't do this in the browser, we get a runtime error.
//         .without_time()
//         .with_ansi(false)
//         .init();

//     tracing::info!("(testing logging) Logging setup");
// }


// /// Blockchain node using Iroh
// #[wasm_bindgen]
// pub struct MinerInstance {
//     bc: BlockChain,
//     mining_block: Option<Block>,
//     eval_request: Arc<Mutex<Option<(Block, Option<bool>)>>>,
//     miner_name: [u8; CONFIG.max_name_length]
// }

// #[wasm_bindgen]
// impl MinerInstance {
//     pub async fn new(miner_name: String, nowait: bool) -> Result<Self, JsError> {
//         let (bc, eval_request, new_block_signal) = BlockChain::new(nowait)
//             .await
//             .map_err(to_js_err)?;
//         Ok(Self {
//             bc,
//             mining_block: None,
//             eval_request,
//             miner_name: parse_miner_name(miner_name)
//         })
//     }

//     pub async fn start_mine(&mut self) -> Result<u32, JsError> {
//         let block = self.bc.start_mine(self.miner_name).await.map_err(to_js_err)?;
//         let seed = block.calc_seed();
//         self.mining_block = Some(block);
//         Ok(seed)
//     }

//     pub async fn submit_mine(&self, seed: u32, solution: Vec<u8>) -> Result<(), JsError> {
//         match self.mining_block {
//             Some(mut block) => {
//                 if block.calc_seed() != seed {
//                     return Err(Error::msg("Wrong seed")).map_err(to_js_err);
//                 }
//                 block.seal(solution);
//                 self.bc.submit_mine(block).map_err(to_js_err).await
//             }
//             None => {
//                 Err(Error::msg("You didn't use start_mine() first")).map_err(to_js_err)
//             }
//         }
//     }

//     pub async fn get_eval_request(&self) -> Result<Vec<u8>, JsError> {
//         let e_request = self.eval_request.lock().await;
//         let (block, valid) = (*e_request).ok_or_else(|| Error::msg("eval request empty")).map_err(to_js_err)?;
//         if valid.is_some() {
//             return Err(Error::msg("Already responded to this request")).map_err(to_js_err)?;
//         }
//         let seed = block.calc_seed();
//         let solution = block.get_solution();

//         let mut result = Vec::new();
//         result.extend(seed.to_le_bytes());
//         result.extend(solution);
//         Ok(result)
//     }

//     pub async fn respond_eval_request(&self, seed: u32, valid: bool) -> Result<(), JsError> {
//         let mut e_request = self.eval_request.lock().await;
//         let (block, _) = (*e_request).ok_or_else(|| Error::msg("eval request empty")).map_err(to_js_err)?;
//         let b_seed = block.calc_seed();
//         if b_seed != seed {
//             return Err(Error::msg("eval request block changed")).map_err(to_js_err);
//         }
//         *e_request = Some((block, Some(valid)));
//         Ok(())
//     }

//     pub async fn get_head_bytes(&self) -> Result<Vec<u8>, JsError> {
//         let block = self.bc.get_head_block_public().await.map_err(to_js_err)?;
//         Ok(block.get_solution())
//     }

//     pub async fn has_new_block(&self) -> bool {
//         self.bc.has_new_block().await
//     }
// }

fn to_js_err(err: impl Into<anyhow::Error>) -> JsError {
    let err: anyhow::Error = err.into();
    JsError::new(&err.to_string())
}