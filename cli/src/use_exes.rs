use std::process::Command;
use std::env;
use n0_snafu::{Result, ResultExt};
use sm64_crypto_shared::Config;
use snafu::whatever;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;
use std::io::{Read, Write};
use std::fs::File;

fn create_info_file(seed: u32, record_mode: u32, config: Config) -> NamedTempFile {
    let info_file = NamedTempFile::new().expect("failed to make temp file");
    let filename = info_file.path();
    {
        let mut file = File::create(filename).expect("failed to create file");

        // Write each u32 as raw bytes
        file.write_all(&seed.to_le_bytes()).expect("failed to write seed");
        file.write_all(&record_mode.to_le_bytes()).expect("failed to write record mode");
        
        // Convert usize to u32 and write as raw bytes
        file.write_all(&(config.max_solution_bytes as u32).to_le_bytes()).expect("failed to write max solution bytes");
        file.write_all(&(config.max_window_length as u32).to_le_bytes()).expect("failed to write window length max");
        file.write_all(&(config.max_random_action as u32).to_le_bytes()).expect("failed to write random action max");
    }

    info_file
}

#[allow(unused)]
pub fn ez_record(seed: u32, starting_bytes: &Vec<u8>, config: Config) -> (Vec<u8>, bool) {
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
    let info_file = create_info_file(seed, 1, config);
    let status = Command::new(sm64_path)
        .arg(filename)
        .arg(info_file.path())
        .spawn().e().expect("").wait().expect("");

    let won: bool = status.success();

    // Read back from the temp file
    let mut solution_bytes = Vec::new();
    solution_bytes_pipe.read_to_end(&mut solution_bytes).expect("");
    (solution_bytes, won)
}

pub fn record(seed: u32, starting_bytes: &Vec<u8>, kill_signal: Arc<Mutex<bool>>, config: Config) -> Result<(Vec<u8>, bool)> {
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
    let info_file = create_info_file(seed, 1, config);
    let mut child = Command::new(sm64_path)
        .arg(filename)
        .arg(info_file.path())
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

pub fn record_loop(seed: u32,  kill_signal: Arc<Mutex<bool>>, config: Config) -> Result<Vec<u8>> {
    let mut solution_bytes: Vec<u8> = Vec::new();
    loop {
        let won;
        (solution_bytes, won) = record(seed, &solution_bytes.clone(), kill_signal.clone(), config)?; // if the kill signal is used, then the ? activates
        // let success = ez_evaluate(seed, &solution_bytes.clone(), 0);

        if won {
            break;
        }
    }
    Ok(solution_bytes)
}

#[allow(unused)]
pub fn ez_record_loop(seed: u32, config: Config) -> Vec<u8> {
    let mut solution_bytes: Vec<u8> = Vec::new();
    loop {
        let won;
        (solution_bytes, won) = ez_record(seed, &solution_bytes.clone(), config);
        // let success = ez_evaluate(seed, &solution_bytes.clone(), 0);

        if won {
            break;
        }
    }
    solution_bytes
}

pub fn ez_evaluate(seed: u32, solution_bytes: &Vec<u8>, headless: bool, config: Config) -> bool {
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let current_directory = exe_path.parent().expect("Failed to get parent directory");
    let sm64_path;

    if headless {
        sm64_path = current_directory.join("sm64_headless.us");
    } else {
        sm64_path = current_directory.join("sm64.us");
    }

    // Spawn `evaluate`, pass seed and fps as args
    let solution_bytes_pipe = NamedTempFile::new().expect("failed to make temp file");
    let filename = solution_bytes_pipe.path();
    {
        let mut file = File::create(filename).expect("");
        file.write_all(solution_bytes).expect("");
    }

    let info_file = create_info_file(seed, 1, config);
    let status = Command::new(sm64_path)
        .arg(filename)
        .arg(info_file.path())
        .spawn().expect("Failed to execute command").wait().expect("");

    let success: bool = status.success();
    success
}