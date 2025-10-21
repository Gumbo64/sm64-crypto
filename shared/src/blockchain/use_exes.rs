use std::process::Command;
use std::env;
use n0_snafu::{Result, ResultExt};
use snafu::whatever;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;
use std::io::{Read, Write};
use std::fs::File;

#[allow(unused)]
pub fn ez_record(seed: &str, starting_bytes: &Vec<u8>) -> (Vec<u8>, bool) {
    // Locate sibling binaries (built in the same target dir as this binary)
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let current_directory = exe_path.parent().expect("Failed to get parent directory");
    let sm64_path = current_directory.join("record");

    let mut solution_bytes_pipe = NamedTempFile::new().expect("failed to make temp file");
    let filename = solution_bytes_pipe.path();
    {
        let mut file = File::create(filename).expect("");
        file.write_all(starting_bytes).expect("");
    }

    let status = Command::new(sm64_path)
        .arg(seed)    
        .arg(filename)
        .arg("1")
        .spawn().e().expect("").wait().expect("");

    let won: bool = status.success();

    // Read back from the temp file
    let mut solution_bytes = Vec::new();
    solution_bytes_pipe.read_to_end(&mut solution_bytes).expect("");
    (solution_bytes, won)
}

pub fn record(seed: &str, starting_bytes: &Vec<u8>, kill_signal: Arc<Mutex<bool>>) -> Result<(Vec<u8>, bool)> {
    // Locate sibling binaries (built in the same target dir as this binary)
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let current_directory = exe_path.parent().expect("Failed to get parent directory");
    let sm64_path = current_directory.join("sm64.us");

    let mut solution_bytes_pipe = NamedTempFile::new().expect("failed to make temp file");
    let filename = solution_bytes_pipe.path();
    {
        let mut file = File::create(filename).expect("");
        file.write_all(starting_bytes).expect("");
    }


    let mut child = Command::new(sm64_path)
        .arg(seed)    
        .arg(filename)
        .arg("1")
        .spawn().e()?;

    // Loop to check for kill signal and child process status
    loop {
        // Check if the kill signal is set to true
        {
            match kill_signal.try_lock() {
                Ok(kill) => {
                    if *kill {
                        // If kill signal is true, kill and return an error
                        child.kill().e()?;
                        whatever!("killed early");
                    }
                }
                Err(_e) => {}
            }

        }

        // Check if the child process has exited
        match child.try_wait() {
            Ok(Some(_)) => {
                // Child process has exited, break the loop
                break;
            }
            Ok(None) => {
                // Child process is still running, wait a bit before checking again
                thread::sleep(Duration::from_millis(100));
            }
            Err(_e) => {
                // An error occurred while trying to check the process status
                whatever!("child error")
            }
        }
    }

    let status = child.wait().expect("Failed to execute command");
    let won: bool = status.success();

    // Read back from the temp file
    let mut solution_bytes = Vec::new();
    solution_bytes_pipe.read_to_end(&mut solution_bytes).e()?;
    Ok((solution_bytes, won))
}

pub fn record_loop(seed: &str,  kill_signal: Arc<Mutex<bool>>) -> Result<Vec<u8>> {
    let mut solution_bytes: Vec<u8> = Vec::new();
    loop {
        let won;
        (solution_bytes, won) = record(seed, &solution_bytes.clone(), kill_signal.clone())?; // if the kill signal is used, then the ? activates
        // let success = ez_evaluate(seed, &solution_bytes.clone(), 0);

        if won {
            break;
        }
    }
    Ok(solution_bytes)
}

#[allow(unused)]
pub fn ez_record_loop(seed: &str) -> Vec<u8> {
    let mut solution_bytes: Vec<u8> = Vec::new();
    loop {
        let won;
        (solution_bytes, won) = ez_record(seed, &solution_bytes.clone());
        // let success = ez_evaluate(seed, &solution_bytes.clone(), 0);

        if won {
            break;
        }
    }
    solution_bytes
}

pub fn ez_evaluate(seed: &str, solution_bytes: &Vec<u8>, fps: i8) -> bool {
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let current_directory = exe_path.parent().expect("Failed to get parent directory");
    let sm64_path;

    if fps > 0 {
        sm64_path = current_directory.join("sm64.us");
    } else {
        sm64_path = current_directory.join("sm64_headless.us");
    }

    // Spawn `evaluate`, pass seed and fps as args
    let solution_bytes_pipe = NamedTempFile::new().expect("failed to make temp file");
    let filename = solution_bytes_pipe.path();
    {
        let mut file = File::create(filename).expect("");
        file.write_all(solution_bytes).expect("");
    }

    let status = Command::new(sm64_path)
        .arg(seed)
        .arg(filename)
        .arg("0")
        .spawn().expect("Failed to execute command").wait().expect("");

    let success: bool = status.success();
    success
}