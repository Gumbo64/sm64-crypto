mod blockchain;

use std::str::FromStr;
pub use blockchain::{Block, GamePad};
use blockchain::{BlockChain, Ticket};
use hex::ToHex;
use iroh_blobs::Hash;
use iroh_gossip::TopicId;
use sm64_binds::{RngConfig, SM64GameGenerator};
use crate::CHAIN_CFG;

use anyhow::{Result, Error};

#[derive(Debug)]
pub struct BlockChainClient {
    bc: BlockChain,
    mining_block: Option<Block>,
    miner_name: String,
    topic_id: TopicId
}

impl BlockChainClient {
    pub async fn new(rom_bytes: Vec<u8>, miner_name: String, ticket_opt: Option<String>) -> Result<Self> {
        if miner_name.len() > CHAIN_CFG.max_name_length {
            return Err(Error::msg("Miner name is too long"));
        }

        let game_gen = SM64GameGenerator::new(rom_bytes)?;

        let ticket = match ticket_opt {
            Some(ticket_str) => {
                Ticket::deserialize(&ticket_str)?
            },
            None => {
                Ticket::new_random()
            }
        };

        let topic_id = ticket.topic_id;
        let bc = BlockChain::new(game_gen, ticket).await?;

        Ok(Self {
            bc,
            topic_id,
            mining_block: None,
            miner_name,
        })
    }

    pub fn get_ticket(&self) -> String {
        let topic_id = self.topic_id;
        let bootstrap = [self.bc.endpoint_id()].into_iter().collect();
        let ticket = Ticket {topic_id, bootstrap};
        ticket.serialize()
    }

    pub async fn start_mine(&mut self) -> Result<(u32, RngConfig)> {
        self.bc.start_mine().await;

        let head = self.bc.get_head_public().await?;
        let block = Block::new(head, self.miner_name.clone())?;

        let seed = block.calc_seed();
        let cfg = block.calc_rng_config();

        self.mining_block = Some(block);

        Ok((seed, cfg))
    }

    pub async fn submit_mine(&mut self, seed: u32, solution: Vec<GamePad>) -> Result<()> {
        match self.mining_block.clone() {
            Some(mut block) => {
                if block.calc_seed() != seed {
                    return Err(Error::msg("The provided seed does not match start_mine()"));
                }

                block.seal(solution)?;
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

    pub async fn get_head_hash(&self) -> Result<String> {
        let head = self.bc.get_head_public().await?;
        Ok(head.hash.encode_hex())
    }

    pub async fn get_block_from_str(&self, hash_str: String) -> Result<Block> {
        let hash = Hash::from_str(&hash_str)?;
        let block = self.bc.get_local_block_public(hash).await?;
        Ok(block)
    }

}