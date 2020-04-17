//! Install packages to test dependencies.

use std::process::Command;

/// Use the binary elm-json to copy behavior of `elm-test install ...`.
pub fn main(packages: Vec<String>) {
    // Recommend direct usage of elm-json instead
    println!("Don't hesitate to try zwilias/elm-json directly instead!");

    // Install packages in test dependencies
    let _ = Command::new("elm-json")
        .arg("install")
        .arg("--test")
        .args(packages)
        .status()
        .expect("Command elm-json failed to start");
}
