mod blockchain;

use std::str::FromStr;

use blockchain::{Block, BlockChain, Ticket, GamePad};
use iroh_blobs::Hash;
use sm64_binds::SM64GameGenerator;
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

#[derive(Debug)]
pub struct BlockChainClient {
    bc: BlockChain,
    mining_block: Option<Block>,
    miner_name: [u8; DEFAULT_CONFIG.max_name_length],
    ticket: Ticket,
}

impl BlockChainClient {
    pub async fn new(miner_name: String, ticket_str: String) -> Result<Self> {
        let game_gen = SM64GameGenerator::from_rom_bytes(include_bytes!("../../baserom.us.z64").to_vec())?;
        let ticket = match Ticket::from_str(&ticket_str) {
            Ok(t) => {
                println!("Ticket SUCCESS");
                Some(t)
            },
            Err(_) => {
                println!("Ticket FAILED");
                None
            },
        };

        let (bc, ticket) = BlockChain::new(game_gen, ticket).await?;
        Ok(Self {
            bc,
            ticket,
            mining_block: None,
            miner_name: parse_miner_name(miner_name),
        })
    }

    pub fn get_ticket(&self) -> String {
        self.ticket.to_string()
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

                let mut solution_pads: Vec<GamePad> = Vec::new();
                for chunk in solution.chunks(4) {
                    if chunk.len() == 4 {
                        // Only create GamePad if there's a complete chunk of 4 bytes
                        let pad = GamePad::from_bytes(chunk);
                        solution_pads.push(pad);
                    } else {
                        eprintln!("Warning: Incomplete chunk ignored: {:?}", chunk);
                    }
                }

                block.seal(solution_pads);
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