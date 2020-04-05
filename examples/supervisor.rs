use std::io::Write;
use std::process::{Command, Stdio};
use std::{thread, time};

/// usage: cargo run --example supervisor
pub fn main() {
    let mut supervisor = Command::new("node")
        .arg("examples/supervisor/supervisor.js")
        .stdin(Stdio::piped())
        .spawn()
        .expect("command failed to start");

    // Helper closure to write to supervisor
    let stdin = supervisor.stdin.as_mut().expect("Failed to open stdin");
    let mut writeln = |msg| {
        stdin.write_all(msg).expect("writeln");
        stdin.write_all(b"\n").expect("writeln");
    };

    // Send multiple rounds of tests (simulate --watch)
    writeln(b"{\"nbTests\": 3, \"runner\": \"./runner.js\"}");
    writeln(b"{\"nbTests\": 4, \"runner\": \"./runner.js\"}");
    writeln(b"{\"nbTests\": 5, \"runner\": \"./runner.js\"}");
    thread::sleep(time::Duration::from_secs(3));
    writeln(b"{\"nbTests\": 6, \"runner\": \"./runner.js\"}");

    // Wait for supervisor child process to end
    supervisor.wait();
}
