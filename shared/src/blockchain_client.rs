mod blockchain;

use blockchain::{Block, BlockChain};
use iroh_blobs::Hash;
use crate::DEFAULT_CONFIG;

use anyhow::{Result, Error};

fn parse_miner_name(s: String) -> [u8; DEFAULT_CONFIG.max_name_length] {
    let mut miner_name: [u8; DEFAULT_CONFIG.max_name_length] = [0; DEFAULT_CONFIG.max_name_length];
    let vec = s;
    assert!(vec.len() <= DEFAULT_CONFIG.max_name_length);
    let copy_length = vec.len().min(DEFAULT_CONFIG.max_name_length);
    miner_name[..copy_length].copy_from_slice(&vec.as_bytes()[..copy_length]);
    miner_name
}

#[derive(Debug, Clone)]
pub struct BlockChainClient {
    bc: BlockChain,
    mining_block: Option<Block>,
    miner_name: [u8; DEFAULT_CONFIG.max_name_length],
}

impl BlockChainClient {
    pub async fn new(miner_name: String, nowait: bool) -> Result<Self> {
        let bc= BlockChain::new(nowait).await?;
        Ok(Self {
            bc,
            mining_block: None,
            miner_name: parse_miner_name(miner_name),
        })
    }

    pub async fn start_mine(&mut self) -> Result<u32> {
        let block = self.bc.start_mine(self.miner_name).await?;
        let seed = block.calc_seed();
        self.mining_block = Some(block);
        Ok(seed)
    }

    pub async fn submit_mine(&mut self, seed: u32, solution: Vec<u8>) -> Result<()> {
        match self.mining_block {
            Some(mut block) => {
                if block.calc_seed() != seed {
                    return Err(Error::msg("The provided seed does not match start_mine()"));
                }
                block.seal(solution);
                match self.bc.submit_mine(block).await {
                    Ok(_) => {
                        self.mining_block = None;
                        Ok(())
                    }
                    Err(e) => Err(e)
                }
            }
            None => {
                Err(Error::msg("Block does not exist or has already been mined. Use start_mine()"))
            }
        }
    }
    pub async fn has_new_block(&self) -> bool {
        self.bc.has_new_block().await
    }
    
    pub async fn get_eval_request(&self) -> Result<(u32, Vec<u8>)> {
        self.bc.get_eval_request().await
    }

    pub async fn respond_eval_request(&self, seed: u32, valid: bool) -> Result<()> {
        self.bc.respond_eval_request(seed, valid).await
    }

    pub async fn get_head(&self) -> Result<Block> {
        self.bc.get_head_block_public().await
    }

    pub async fn get_block(&self, hash: Hash) -> Result<Block> {
        self.bc.get_local_block_public(hash).await
    }

    pub async fn get_block_from_str(&self, hash: String) -> Result<Block> {
        let hash_bytes = hash.as_bytes();
        let l1 = hash_bytes.len();
        let l2 = Hash::EMPTY.as_bytes().len();
        if l1 != l2 {
            println!("hash lengths: {} {}\n", l1, l2);
            return Err(Error::msg("Provided hash is of the wrong length, might be whitespace"));
        }
        let mut array: [u8; 32] = [0u8; 32];
        array[..hash_bytes.len()].copy_from_slice(hash_bytes);

        let block = self.get_block(Hash::from_bytes(array)).await?;
        Ok(block)
    }

}