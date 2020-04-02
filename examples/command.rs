use std::process::Command;

/// usage: cargo run --example command -- elm --help
pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    Command::new(&args[1])
        .args(&args[2..])
        .spawn()
        // see also status() and output()
        // https://doc.rust-lang.org/std/process/struct.Command.html
        .expect("command failed to start");
}
