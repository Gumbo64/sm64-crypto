#[derive(Debug, Clone, Copy)]
pub struct ChainConfig {
    pub max_name_length: usize,
    pub max_solution_time: usize,
}
impl ChainConfig {
    const fn default() -> Self {
        Self {
            max_name_length: 64,
            max_solution_time: 10 * 60 * 30,
        }
    }
}

pub const CHAIN_CFG: ChainConfig = ChainConfig::default();
