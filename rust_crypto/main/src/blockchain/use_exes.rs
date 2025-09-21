use std::process::Command;
use std::env;
use n0_snafu::{Result, ResultExt};
use snafu::whatever;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::thread;
use std::time::Duration;


pub fn ez_record(seed: &str, starting_bytes: &Vec<u8>) -> Vec<u8> {
    // Locate sibling binaries (built in the same target dir as this binary)
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let current_directory = exe_path.parent().expect("Failed to get parent directory");
    let record_command_path = current_directory.join("record");

    let output = Command::new(record_command_path)
        .arg(seed)            
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(&starting_bytes)?;
            }
            child.wait_with_output()
        })
        .expect("Failed to execute command");

    output.stdout.clone()
}
pub fn record(seed: &str, starting_bytes: &Vec<u8>, kill_signal: Arc<Mutex<bool>>) -> Result<Vec<u8>> {
    // Locate sibling binaries (built in the same target dir as this binary)
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let current_directory = exe_path.parent().expect("Failed to get parent directory");
    let record_command_path = current_directory.join("record");

    let mut child = Command::new(record_command_path)
        .arg(seed)            
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            // take() means that child.stdin is dropped, which tells ./record that it doesn't have to wait for further input
            if let Some(stdin) = child.stdin.take().as_mut() {
                stdin.write_all(&starting_bytes)?;
            }
            Ok(child)
        }).e()?;


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

    // Collect the output after the child process has exited
    let output = child.wait_with_output().expect("Failed to execute command");

    Ok(output.stdout.clone())
}

pub fn record_loop(seed: &str,  kill_signal: Arc<Mutex<bool>>) -> Result<Vec<u8>> {
    let mut solution_bytes: Vec<u8> = Vec::new();
    loop {
        solution_bytes = record(seed, &solution_bytes.clone(), kill_signal.clone())?; // if the kill signal is used, then the ? activates
        let success = ez_evaluate(seed, &solution_bytes.clone(), 0);

        if success {
            break;
        }
    }
    Ok(solution_bytes)
}

pub fn ez_record_loop(seed: &str) -> Vec<u8> {
    let mut solution_bytes: Vec<u8> = Vec::new();
    loop {
        solution_bytes = ez_record(seed, &solution_bytes.clone());
        let success = ez_evaluate(seed, &solution_bytes.clone(), 0);

        if success {
            break;
        }
    }
    solution_bytes
}

pub fn ez_evaluate(seed: &str, solution_bytes: &Vec<u8>, fps: i8) -> bool {
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let current_directory = exe_path.parent().expect("Failed to get parent directory");
    let evaluate_command_path = current_directory.join("evaluate");

    // Spawn `evaluate`, pass seed and fps as args, and stream replay bytes via stdin
    let output = Command::new(evaluate_command_path)
            .arg(seed)
            .arg(fps.to_string())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                // Pipe the recorded replay into `evaluate`'s stdin
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(&solution_bytes)?;
                }
                // Wait for `evaluate` to finish and capture its output
                child.wait_with_output()
            })
            .expect("Failed to execute command");

    let success: bool = &output.stdout == &"true".as_bytes().to_vec();
    success
}

use std::fs;
use std::io;
use std::path::Path;
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
