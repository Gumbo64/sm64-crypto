
use sm64_binds::RandomConfig;
#[derive(Debug, Clone, Copy)]
pub struct ChainConfig {
    pub max_name_length: usize,
    pub max_solution_time: usize,
    pub random_config: RandomConfig,
}
impl ChainConfig {
    const fn new(max_name_length: usize, max_solution_time: usize) -> Self {
        Self {
            max_name_length,
            max_solution_time,
            random_config: RandomConfig::default()
        }
    }
}

const MAX_NAME_LENGTH: usize = 64;
const MAX_SOLUTION_TIME: usize = 10 * 60 * 30; // 10 minutes

// const MAX_SOLUTION_BYTES: usize = MAX_SOLUTION_TIME * 30 * 4; // seconds * fps * (bytes per frame) 

pub const DEFAULT_CONFIG: ChainConfig = ChainConfig::new(
    MAX_NAME_LENGTH, 
    MAX_SOLUTION_TIME,
);
