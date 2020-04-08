use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// usage: cargo run --example elm-json-bin
pub fn main() {
    let output = Command::new("elm-json")
        .arg("solve")
        .arg("--test")
        .arg("--extra")
        .arg("elm/core")
        .arg("elm/json")
        .arg("elm/time")
        .arg("elm/random")
        .arg("--")
        .arg(Path::new("examples/elm-json/elm.json"))
        // stdio config
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .expect("command failed to start");
    std::io::stdout().write_all(&output.stdout).unwrap();
}
