use sm64_so::{buttons_to_int, eval_metric, Replay, SM64Game, StatefulInputGenerator};

use device_query::{DeviceQuery, DeviceState, Keycode};
use std::io::{self, Write, Read};
use std::time::Instant;
use std::time::Duration;
use std::env;

struct Config {
    speed: f32,
    goback_amount: usize,
    reset: bool,
    goback: bool,
}

fn get_inputs(held_keys: &Vec<Keycode>) -> (u16, i8, i8) {
    let mut stick_x = 0;
    let mut stick_y = 0;
    if held_keys.contains(&Keycode::A) {
        stick_x = -80;
    } else if held_keys.contains(&Keycode::D) {
        stick_x = 80;
    }
    if held_keys.contains(&Keycode::W) {
        stick_y = 80;
    } else if held_keys.contains(&Keycode::S) {
        stick_y = -80;
    }

    let button = buttons_to_int(
        held_keys.contains(&Keycode::I),
        held_keys.contains(&Keycode::J),
        false,
        false,
        held_keys.contains(&Keycode::O),
        held_keys.contains(&Keycode::Enter),
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
    );

    (button, stick_x, stick_y)
}

fn set_config(held_keys: Vec<Keycode>, config: &mut Config) {
    if held_keys.contains(&Keycode::Space) {
        config.speed = if held_keys.contains(&Keycode::LShift) { 10.0 } else { 0.1 };
    } else {
        config.speed = 1.0;
    }
    if held_keys.contains(&Keycode::Q) {
        config.reset = true;
    }
    if held_keys.contains(&Keycode::R) {
        config.goback = true;
    }
}

pub fn record_replay(seed: &str, starting_bytes: Vec<u8>) -> (Vec<u8>, bool) {
    let mut solution_bytes: Vec<u8> = starting_bytes.clone();

    let sm64_game = SM64Game::new(false).unwrap();
    let mut input_gen = StatefulInputGenerator::new(seed);
    let mut won = false;

    // Play the starting bytes
    let replay = Replay::new(starting_bytes, false);

    for (button, stick_x, stick_y) in replay {
        // println!("{} {} {}", button, stick_x, stick_y);
        if !input_gen.validate_action(&sm64_game, button, stick_x, stick_y) {
            println!("Invalid input detected: {}, {}, {}", button, stick_x, stick_y);
            break;
        }
        sm64_game.step_game(1, stick_x, stick_y, button);
        if eval_metric(&sm64_game) {
            won = true;
            break;
        }
    }

    // Start recording
    let mut config = Config {
        speed: 1.0,
        goback_amount: 120,
        reset: false,
        goback: false
    };
    let ds = DeviceState::new();
    let mut last_time: Instant = Instant::now();
    
    loop {
        
        let held_keys = ds.get_keys();

        let (mut b, mut x, mut y) = get_inputs(&held_keys);
        set_config(held_keys, &mut config);


        let (action, random_tick) = input_gen.get_iter(&sm64_game);
        if random_tick {
            (b, x, y) = action;
        }

        // Split the u16 button into two bytes (big-endian)
        let high_byte = (b >> 8) as u8; // Get the high byte
        let low_byte = (b & 0xFF) as u8; // Get the low byte

        solution_bytes.push(high_byte);
        solution_bytes.push(low_byte);
        solution_bytes.push(x as u8);
        solution_bytes.push(y as u8);

        sm64_game.step_game(1, x, y, b);
        if eval_metric(&sm64_game) {
            won = true;
            break;
        }

        if config.goback {
            let new_length = solution_bytes.len().saturating_sub(4 * config.goback_amount);
            solution_bytes.truncate(new_length);
            break;
        }

        if config.reset {
            solution_bytes = Vec::new();
            break;
        }
        let elapsed = last_time.elapsed();
        let dur = Duration::from_secs_f64(1.0 / (30.0 * config.speed) as f64);
        if elapsed < dur {
            std::thread::sleep(dur - elapsed);
        }
        last_time = Instant::now();
    }

    (solution_bytes, won)
}

fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if the expected number of arguments is provided
    if args.len() < 2 {
        eprintln!("Usage: {} <data>", args[0]);
        std::process::exit(1);
    }

    let seed: &str = &args[1];

    let mut starting_bytes: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut starting_bytes).expect("Failed to read data");

    let (solution_bytes, _won) = record_replay(seed, starting_bytes);
    // if !won {
    //     return;
    // }

    let mut stdout = io::stdout();
    stdout.write_all(solution_bytes.as_slice()).expect("Failed to write to stdout");
}
