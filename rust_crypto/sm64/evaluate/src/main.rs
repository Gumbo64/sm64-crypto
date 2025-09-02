use sm64_so::{SM64Game, eval_metric, StatefulInputGenerator, Replay};

use std::time::Instant;
use std::time::Duration;

pub fn evaluate_replay(seed: &str, solution_bytes: Vec<u8>, fps: u32) -> bool {
    let step_time = if fps > 0 { Duration::from_secs_f64(1.0 / fps as f64) } else { Duration::from_secs(0) };
    let headless = fps == 0;

    // let step_time = Duration::from_secs_f64(0.016667);
    // let headless = false;

    let sm64_game = SM64Game::new(headless).unwrap();
    let mut input_gen = StatefulInputGenerator::new(seed);
    let mut won = false;

    let replay = Replay::new(solution_bytes, false);

    let mut last_time = Instant::now();

    for (button, stick_x, stick_y) in replay {
        if !input_gen.validate_action(&sm64_game, button, stick_x, stick_y) {
            println!("Invalid input detected: {}, {}, {}", button, stick_x, stick_y);
            break;
        }

        sm64_game.step_game(1, stick_x, stick_y, button);
        if eval_metric(&sm64_game) {
            won = true;
            break;
        }

        // Sleep for the calculated step time
        let elapsed = last_time.elapsed();
        if elapsed < step_time {
            std::thread::sleep(step_time - elapsed);
        }
        last_time = Instant::now();
    }

    if !won {
        sm64_game.step_game(1, 0, 0, 0);
        won = eval_metric(&sm64_game);
        println!("Final evaluation: {}", won);
    }

    won
}

use std::env;
use std::io::{self, Read};

fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if the expected number of arguments is provided
    if args.len() < 3 {
        eprintln!("Usage: {} <data>", args[0]);
        std::process::exit(1);
    }
    
    // The first argument is the program name, the second is the input data
    let seed: &str = &args[1];
    let fps: u32 = args[2].parse().expect("Failed to convert fps to u32");

    let mut solution_bytes: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut solution_bytes).expect("Failed to read data");
    
    println!("{}", evaluate_replay(seed, solution_bytes, fps)); // "returns" true or false
}