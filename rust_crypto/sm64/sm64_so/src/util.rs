use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::f32;

use super::constants::BUTTONS;

use super::game::SM64Game;

use std::fs;
use std::io;
use std::path::Path;


/// Removes all `.tmp.so` files from the specified directory.
/// Returns the number of files removed or an error.
pub fn remove_tmp_so_files<P: AsRef<Path>>(dir: P) -> io::Result<()> {
    let dir = dir.as_ref().join("tmp_so");
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "so" {
                if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                    if fname.ends_with(".tmp.so") {
                        fs::remove_file(&path)?;
                    }
                }
            }
        }
    }
    Ok(())
}


pub fn buttons_to_int(
    button_a: bool,
    button_b: bool,
    button_l: bool,
    button_r: bool,
    button_z: bool,
    button_start: bool,
    button_du: bool,
    button_dl: bool,
    button_dr: bool,
    button_dd: bool,
    button_cu: bool,
    button_cl: bool,
    button_cr: bool,
    button_cd: bool,
) -> u16 {
    let mut button: u16 = 0;
    if button_a { button |= BUTTONS::A_BUTTON; }
    if button_b { button |= BUTTONS::B_BUTTON; }
    if button_l { button |= BUTTONS::L_TRIG; }
    if button_r { button |= BUTTONS::R_TRIG; }
    if button_z { button |= BUTTONS::Z_TRIG; }
    if button_start { button |= BUTTONS::START_BUTTON; }
    if button_du { button |= BUTTONS::U_JPAD; }
    if button_dl { button |= BUTTONS::L_JPAD; }
    if button_dr { button |= BUTTONS::R_JPAD; }
    if button_dd { button |= BUTTONS::D_JPAD; }
    if button_cu { button |= BUTTONS::U_CBUTTONS; }
    if button_cl { button |= BUTTONS::L_CBUTTONS; }
    if button_cr { button |= BUTTONS::R_CBUTTONS; }
    if button_cd { button |= BUTTONS::D_CBUTTONS; }
    button
}

// pub struct InputGenerator {
//     rng: StdRng,
// }

// impl InputGenerator {
//     pub fn new(seed: &str) -> Self {
//         // Create a seeded RNG from string seed
//         let seed_bytes = {
//             let mut b = [0u8; 32];
//             let s = seed.as_bytes();
//             for i in 0..b.len().min(s.len()) {
//                 b[i] = s[i];
//             }
//             b
//         };
//         let rng = StdRng::from_seed(seed_bytes);
//         Self { rng }
//     }

//     pub fn generate_input(&mut self) -> (u16, i8, i8) {
//         let button: u16 = buttons_to_int(
//             self.rng.random_bool(0.5),
//             self.rng.random_bool(0.5),
//             false, // L
//             false, // R
//             self.rng.random_bool(0.2),
//             false, // Start
//             false, // DU
//             false, // DL
//             false, // DR
//             false, // DD
//             false, // CU
//             false, // CL
//             false, // CR
//             false, // CD
//         );

//         let stick_x: i8 = self.rng.random_range(-80..=80);
//         let stick_y: i8 = self.rng.random_range(-80..=80);
//         (button, stick_x, stick_y)
//     }

//     pub fn validate_input(&mut self, button: u16, stick_x: i8, stick_y: i8) -> bool {
//         let (b, x, y) = self.generate_input();
//         b == button && x == stick_x && y == stick_y
//     }

//     pub fn is_random_tick(&mut self) -> bool {
//         self.rng.random_bool(0.05)
//     }

//     pub fn fake_iter(&mut self) {
//         if self.is_random_tick() {
//             let _ = self.generate_input();
//         }
//     }
// }

pub struct StatefulInputGenerator {
    rng: StdRng,
    rng_window_length_max: u16,
    rng_window_cur_amount: u16,

    rng_random_action_max: u16,
    rng_random_action_amount: u16
}

impl StatefulInputGenerator {
    pub fn new(seed: &str) -> Self {
        // Create a seeded RNG from string seed
        let seed_bytes = {
            let mut b = [0u8; 32];
            let s = seed.as_bytes();
            for i in 0..b.len().min(s.len()) {
                b[i] = s[i];
            }
            b
        };
        let rng = StdRng::from_seed(seed_bytes);
        Self { 
            rng, 
            rng_window_length_max: 100,
            rng_random_action_max: 5,

            rng_window_cur_amount: 0,
            rng_random_action_amount: 0
        }

    }
    pub fn validate_action(&mut self, game: &SM64Game, button: u16, stick_x: i8, stick_y: i8) -> bool {
        let (action, random) = self.get_iter(game);
        if !random {
            return true;
        }
        let (b, x, y) = action;

        b == button && x == stick_x && y == stick_y
    }

    pub fn get_iter(&mut self, game: &SM64Game) -> ((u16, i8, i8), bool) {
        self.update_rng(game);
        if self.is_random_action() {
            let action = self.generate_action();
            return (action, true)
        }
        ((0,0,0), false)
    }

    fn make_rng(seed: &str) -> StdRng {
        let seed_bytes = {
            let mut b = [0u8; 32];
            let s = seed.as_bytes();
            for i in 0..b.len().min(s.len()) {
                b[i] = s[i];
            }
            b
        };
        return StdRng::from_seed(seed_bytes);
    }

    fn generate_action(&mut self) -> (u16, i8, i8) {
        let button: u16 = buttons_to_int(
            self.rng.random_bool(0.5),
            self.rng.random_bool(0.5),
            false, // L
            false, // R
            self.rng.random_bool(0.2),
            false, // Start
            false, // DU
            false, // DL
            false, // DR
            false, // DD
            false, // CU
            false, // CL
            false, // CR
            false, // CD
        );

        let stick_x: i8 = self.rng.random_range(-80..=80);
        let stick_y: i8 = self.rng.random_range(-80..=80);
        (button, stick_x, stick_y)
    }

    fn is_random_action(&mut self) -> bool {

        if self.rng_window_cur_amount == 0 {
            // Reset the window length and random action amount
            self.rng_window_cur_amount = self.rng_window_length_max;
            self.rng_random_action_amount = self.rng_random_action_max;
        }

        let prob_random: f64 = self.rng_random_action_amount as f64 / self.rng_window_cur_amount as f64;
        let is_random = self.rng.random_range(0.0..1.0) < prob_random;
        
        if is_random {
            self.rng_random_action_amount -= 1;
        }
        self.rng_window_cur_amount -= 1;

        is_random
    }

    fn update_rng(&mut self, game: &SM64Game) {
        let state = game.get_mario_state();

        // let game_numbers = [state.pos, state.faceAngle]; just use state pos for the moment.
        // factor in angle etc later

        // The next RNG is influenced by
        // 1. the current game state
        let game_str: String = state.pos.iter()
            .map(|&num| num.to_string()) // Convert each f32 to String
            .collect::<Vec<String>>() // Collect into a Vec<String>
            .join(" "); // Join

        // 2. the old RNG
        let prev_rng_str: String = self.rng.random_range(0..8192).to_string();

        let total_seed = game_str.clone() + &prev_rng_str.clone();
        self.rng = StatefulInputGenerator::make_rng(&total_seed);
    }


}


fn distance(pos: &[f32; 3], goal: &[f32; 3]) -> f32 {
    let dx = pos[0] - goal[0];
    let dy = pos[1] - goal[1];
    let dz = pos[2] - goal[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn eval_metric(game: &SM64Game) -> bool {
    let goal = [-153.0_f32, 840.0, -356.0];
    let m = game.get_mario_state();

    distance(&m.pos, &goal) < 300.0
}

// pub fn eval_metric_bob(game: &SM64Game) -> bool {
//     let m = game.get_mario_state();
//     let info = game.get_game_info();

//     let is_static = m.vel[0] == 0.0 && m.vel[1] == 0.0 && m.vel[2] == 0.0;
//     let is_on_bob_mountain_peak = (m.pos[1] >= 4292.8 && m.pos[1] <= 4294.2) && info.courseNum == 1;

//     is_on_bob_mountain_peak && is_static
// }

// pub fn eval_metric_credits(game: &SM64Game) -> bool {
//     let info = game.get_game_info();
//     info.inCredits
// }
