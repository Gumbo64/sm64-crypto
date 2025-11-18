use std::u128;
use bytes::Bytes;
use anyhow::{Result, Error};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use iroh_blobs::Hash;
use chrono::{DateTime, Utc};

use crate::DEFAULT_CONFIG;
use sm64_binds::GamePad;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Block {
    pub(crate) prev_hash: Hash,
    pub(crate) block_height: u128,
    pub(crate) timestamp: DateTime<Utc>,

    #[serde(with = "serde_arrays")]
    pub(crate) miner_name: [u8; DEFAULT_CONFIG.max_name_length],
    #[serde(with = "serde_arrays")]
    solution_bytes: [GamePad; DEFAULT_CONFIG.max_solution_time],
}

impl Block {
    pub fn new(block_head: BlockHead, miner_name: [u8; DEFAULT_CONFIG.max_name_length]) -> Self {
        let prev_hash = block_head.hash;
        let block_height = block_head.height.wrapping_add(1);
        
        let timestamp = Utc::now();
        let tmp_solution_bytes: [GamePad; DEFAULT_CONFIG.max_solution_time] = [GamePad::default(); DEFAULT_CONFIG.max_solution_time];
        Block {prev_hash, block_height, timestamp, miner_name, solution_bytes:tmp_solution_bytes}
    }
    pub fn seal(&mut self, solution_vec: Vec<GamePad>) -> Result<()> {
        if solution_vec.len() > DEFAULT_CONFIG.max_solution_time {
            return Err(Error::msg("Solution is too long"));
        }

        let copy_length = solution_vec.len();
        self.solution_bytes[..copy_length].copy_from_slice(&solution_vec[..copy_length]);
        Ok(())
    }

    pub fn calc_seed(&self) -> u32 {
        // perhaps use a different hashing algorithm later
        let mut hasher = Sha256::new();

        let mut x = format!("{}", self.prev_hash);
        hasher.update(x);

        x = format!("{}", self.block_height);
        hasher.update(x);

        x = format!("{}", self.timestamp);
        hasher.update(x);

        x = format!("{:?}", self.miner_name);
        hasher.update(x);

        let result = hasher.finalize();
        // hex::encode(result) // Convert the hash to a hex string
        let hash_bytes = &result[..4];
        u32::from_be_bytes(hash_bytes.try_into().expect("slice with incorrect length"))
    }
    pub fn get_solution(&self) -> Vec<GamePad> {
        self.solution_bytes.to_vec().clone()
    }

    pub fn encode(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(&self)?.into())
    }

    pub fn decode(bytes: &[u8]) -> Result<Block> {
        Ok(postcard::from_bytes(bytes)?)
    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockHead {
    pub hash: Hash,
    pub height: u128,
}
impl BlockHead {
    pub fn no_blocks(&self) -> bool {
        let def = BlockHead::default();
        self.hash == def.hash && self.height == def.height
    }

    pub fn encode(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(&self)?.into())
    }

    pub fn decode(bytes: &[u8]) -> Result<BlockHead> {
        Ok(postcard::from_bytes(bytes)?)
    }
    
}
impl Default for BlockHead {
    fn default() -> Self {
        BlockHead {
            hash: Hash::EMPTY,
            height: u128::MAX,
        }
    }
}