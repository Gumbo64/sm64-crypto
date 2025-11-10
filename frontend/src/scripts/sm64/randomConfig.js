const MAX_NAME_LENGTH = 64;
const MAX_SOLUTION_TIME = 600; // 600 seconds = 10 minutes
const MAX_WINDOW_LENGTH = 100;
const MAX_RANDOM_ACTION = 5;
const A_PROB = 0.5;
const B_PROB = 0.5;
const Z_PROB = 0.2;

// const MAX_SOLUTION_BYTES = MAX_SOLUTION_TIME * 30 * 4; // seconds * fps * (bytes per frame) 

const DEFAULT_CONFIG = make_config(MAX_NAME_LENGTH, MAX_SOLUTION_TIME, MAX_WINDOW_LENGTH, MAX_RANDOM_ACTION, A_PROB, B_PROB, Z_PROB)

function make_config(max_name_length, max_solution_time, max_window_length, max_random_action, A_prob, B_prob, Z_prob) {
    return {
        "max_name_length": max_name_length,
        "max_solution_bytes": max_solution_time * 30 * 4,
        "max_solution_time": max_solution_time,
        "max_window_length": max_window_length,
        "max_random_action": max_random_action,
        "A_prob": A_prob,
        "B_prob": B_prob,
        "Z_prob": Z_prob,
    }
}

export {DEFAULT_CONFIG, make_config};