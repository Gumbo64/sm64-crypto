#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub max_name_length: usize,
    pub max_solution_bytes: usize,
    pub max_window_length: usize,
    pub max_random_action: usize,
}
impl Config {
    const fn new(max_name_length: usize, max_solution_bytes: usize, max_window_length: usize, max_random_action: usize) -> Self {
        Self {
            max_name_length,
            max_solution_bytes,
            max_window_length,
            max_random_action,
        }
    }
}

const MAX_NAME_LENGTH: usize = 64;
const MAX_SOLUTION_TIME: usize = 600; // 600 seconds = 10 minutes
const MAX_WINDOW_LENGTH: usize = 100;
const MAX_RANDOM_ACTION: usize = 5;

const MAX_SOLUTION_BYTES: usize = MAX_SOLUTION_TIME * 30 * 4; // seconds * fps * (bytes per frame) 

pub const DEFAULT_CONFIG: Config = Config::new(
    MAX_NAME_LENGTH, 
    MAX_SOLUTION_BYTES, 
    MAX_WINDOW_LENGTH, 
    MAX_RANDOM_ACTION,
);
