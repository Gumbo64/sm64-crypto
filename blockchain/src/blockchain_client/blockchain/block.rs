use std::u128;
use bytes::Bytes;
use anyhow::{Result, Error};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use iroh_blobs::Hash;
use chrono::{DateTime, Utc};

use crate::CHAIN_CFG;
use sm64_binds::{GamePad, RngConfig};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub prev_hash: Hash,
    pub block_height: u128,
    pub timestamp: DateTime<Utc>,

    pub miner_name: String,
    pub solution: Vec<GamePad>,
}

impl Block {
    pub fn new(block_head: BlockHead, miner_name: String) -> Result<Self> {
        if miner_name.len() > CHAIN_CFG.max_name_length {
            return Err(Error::msg("Miner name is too long"));
        }

        let prev_hash = block_head.hash;
        let block_height = block_head.height.wrapping_add(1);
        
        let timestamp = Utc::now();
        Ok(Block {prev_hash, block_height, timestamp, miner_name, solution:Vec::new()})
    }
    
    pub fn seal(&mut self, solution_vec: Vec<GamePad>) -> Result<()> {
        if solution_vec.len() > CHAIN_CFG.max_solution_time {
            return Err(Error::msg("Solution is too long"));
        }
        self.solution = solution_vec;
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

    pub fn calc_rng_config(&self) -> RngConfig {
        // might be calculated later using block difficulty etc
        return RngConfig::default();
    }

    pub fn encode(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(&self)?.into())
    }

    pub fn decode(bytes: &[u8]) -> Result<Block> {
        let block: Block = postcard::from_bytes(bytes)?;
        if block.miner_name.len() > CHAIN_CFG.max_name_length {
            return Err(Error::msg("Miner name is too long"));
        };
        if block.solution.len() > CHAIN_CFG.max_solution_time {
            return Err(Error::msg("Solution is too long"));
        }
        Ok(block)
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