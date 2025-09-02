// use std::fs;
use std::process::Command;
use std::env;

fn main() {
    // Locate sibling binaries (built in the same target dir as this binary)
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let current_directory = exe_path.parent().expect("Failed to get parent directory");
    let record_command_path = current_directory.join("record");
    let evaluate_command_path = current_directory.join("evaluate");

    // Seed controlling deterministic game/replay generation
    let seed = "my_seed";

    // Run the `record` helper to produce a replay; capture its stdout bytes
    let output = Command::new(record_command_path)
        .arg(seed) // pass the same seed to `record`
        .output() // run and collect stdout/stderr
        .expect("Failed to execute command");

    // Extract the replay/m64 payload from `record`'s stdout
    let solution_bytes = output.stdout.clone();

    // Target FPS for evaluation (affects simulation timing)
    // let fps = 30;
    let fps = 0;


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

    if success {
        println!("Hooray! it works");
    } else {
        println!("It failed bro its over");
    }
    
            
    
}
