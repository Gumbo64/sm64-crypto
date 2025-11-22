
use anyhow::Result;

use tracing::{level_filters::LevelFilter};
use tracing_subscriber_wasm::MakeConsoleWriter;
use wasm_bindgen::{JsError, prelude::wasm_bindgen};
use hex::ToHex;
use sm64_blockchain::{BlockChainClient, GamePad, Block, RngConfig, CHAIN_CFG};

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_writer(
            // To avoide trace events in the browser from showing their JS backtrace
            MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG),
        )
        // If we don't do this in the browser, we get a runtime error.
        .without_time()
        .with_ansi(false)
        .init();

    // tracing::info!("(testing logging) Logging setup");
}

#[wasm_bindgen]
pub struct GamePadWeb {
    button: u16,
    stick_x: i8,
    stick_y: i8,
}

#[wasm_bindgen]
impl GamePadWeb {
    pub fn new(button: u16, stick_x: i8, stick_y: i8) -> Result<Self, JsError> {
        Ok(Self { button, stick_x, stick_y})
    }
}

#[wasm_bindgen]
pub struct RngAndSeedWeb(RngConfig, u32);
#[wasm_bindgen]
impl RngAndSeedWeb {
    #[wasm_bindgen(getter)]
    pub fn seed(&self) -> u32 {self.1}
    #[wasm_bindgen(getter)]
    pub fn window_length(&self) -> u32 {self.0.window_length}
    #[wasm_bindgen(getter)]
    pub fn random_amount(&self) -> u32 {self.0.random_amount}
    #[wasm_bindgen(getter)]
    pub fn random_burst_length(&self) -> u32 {self.0.random_burst_length}
    #[wasm_bindgen(getter)]
    pub fn a_prob(&self) -> f32 {self.0.a_prob}
    #[wasm_bindgen(getter)]
    pub fn b_prob(&self) -> f32 {self.0.b_prob}
    #[wasm_bindgen(getter)]
    pub fn z_prob(&self) -> f32 {self.0.z_prob}
}


#[wasm_bindgen]
pub struct LightBlock {
    prev_hash: String,
    block_height: u128,
    timestamp: String,
    miner_name: String,
}

impl LightBlock {
    fn from_block(block: Block) -> Self {
        Self {
            prev_hash: block.prev_hash.encode_hex(),
            block_height: block.block_height,
            timestamp: block.timestamp.to_string(),
            miner_name: block.miner_name,
        }
    }
}
/// Blockchain node using Iroh
#[wasm_bindgen]
pub struct BlockChainClientWeb {
    client: BlockChainClient
}

#[wasm_bindgen]
impl BlockChainClientWeb {
    pub async fn new(rom_bytes: Vec<u8>, miner_name: String, ticket_str: String) -> Result<Self, JsError> {
        let ticket_opt = match ticket_str.len() == 0 {
            true => None,
            false => Some(ticket_str),
        };

        let client = BlockChainClient::new(rom_bytes, miner_name, ticket_opt)
            .await
            .map_err(to_js_err)?;

        Ok(Self {client})
    }

    pub fn get_ticket(&self) -> Result<String, JsError> {
        Ok(self.client.get_ticket())
    }

    pub async fn start_mine(&mut self) -> Result<RngAndSeedWeb, JsError> {
        let (seed, rng_config) = self.client.start_mine().await.map_err(to_js_err)?;
        Ok(RngAndSeedWeb(rng_config, seed))
    }

    pub async fn submit_mine(&mut self, seed: u32, solution: Vec<GamePadWeb>) -> Result<(), JsError> {
        // let mut solution_pads: Vec<GamePad> = Vec::new();
        // for chunk in solution.chunks(4) {
        //     if chunk.len() == 4 {
        //         // Only create GamePad if there's a complete chunk of 4 bytes
        //         let pad = GamePad::from_bytes(chunk);
        //         solution_pads.push(pad);
        //     } else {
        //         einfo!("Warning: Incomplete chunk ignored: {:?}", chunk);
        //     }
        // }

        let mut solution_pads: Vec<GamePad> = Vec::new();
        for webpad in solution {
            let pad = GamePad::new(webpad.button, webpad.stick_x, webpad.stick_y);
            solution_pads.push(pad);
        }

        self.client.submit_mine(seed, solution_pads).await.map_err(to_js_err)
    }

    pub async fn has_new_block(&self) -> bool {
        self.client.has_new_block().await
    }

    pub async fn get_head_hash(&self) -> Result<String, JsError> {
        let head_hash = self.client.get_head_hash().await.map_err(to_js_err)?;
        Ok(head_hash)
    }

    pub async fn get_light_block(&self, hash_str: String) -> Result<LightBlock, JsError> {
        let block = self.client.get_block_from_str(hash_str).await.map_err(to_js_err)?;
        Ok(LightBlock::from_block(block))
    }

    pub fn get_max_name_length() -> usize {
        CHAIN_CFG.max_name_length
    }
    pub fn get_max_solution_time() -> usize {
        CHAIN_CFG.max_solution_time
    }

}

fn to_js_err(err: impl Into<anyhow::Error>) -> JsError {
    let err: anyhow::Error = err.into();
    JsError::new(&err.to_string())
}