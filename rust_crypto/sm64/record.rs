use super::sm64_so;
use sm64_so::{SM64Game, eval_metric, StatefulInputGenerator, Replay};

use std::time::Instant;
use std::time::Duration;

use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::collections::HashSet;
use std::io::{self, Write};

#[derive(Default)]
struct Config {
    speed: f32,
    goback_amount: usize,
    reset: bool,
    goback: bool,
}

fn get_inputs(held_keys: &HashSet<String>) -> (u16, i8, i8) {
    let mut stick_x = 0;
    let mut stick_y = 0;
    if held_keys.contains("a") {
        stick_x = -80;
    } else if held_keys.contains("d") {
        stick_x = 80;
    }
    if held_keys.contains("w") {
        stick_y = 80;
    } else if held_keys.contains("s") {
        stick_y = -80;
    }
    let button = (held_keys.contains("i") as u16) << 0
        | (held_keys.contains("j") as u16) << 1
        | (held_keys.contains("o") as u16) << 2
        | (held_keys.contains("enter") as u16) << 3;
    (button, stick_x, stick_y)
}

fn set_config(held_keys: &HashSet<String>, config: &mut Config) {
    if held_keys.contains("space") {
        config.speed = if held_keys.contains("shift") { 10.0 } else { 0.1 };
    }
    if held_keys.contains("q") {
        config.reset = true;
    }
    if held_keys.contains("r") {
        config.goback = true;
    }
}

pub fn record_replay(seed: &str) -> (Vec<u8>, bool) {
    let mut solution_bytes: Vec<u8> = Vec::new();
    let mut won: bool = false;





    (solution_bytes, won)

}