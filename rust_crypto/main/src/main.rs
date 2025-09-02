mod use_exes;
use use_exes::{ez_record_loop};

fn main() {
    let seed = "yeah cuz";

    let solution_bytes = ez_record_loop(seed);

    println!("{}\n", solution_bytes.len());
}
