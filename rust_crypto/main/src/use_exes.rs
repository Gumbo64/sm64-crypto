use std::process::Command;
use std::env;

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

pub fn ez_record_loop(seed: &str) -> Vec<u8> {
    let mut solution_bytes: Vec<u8> = Vec::new();
    loop {
        solution_bytes = ez_record(seed, &solution_bytes.clone());
        let success = ez_evaluate(seed, &solution_bytes.clone(), 0);

        if success {
            println!("Hooray! it works");
            break;
        } else {
            println!("It failed bro go back cuz");
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

