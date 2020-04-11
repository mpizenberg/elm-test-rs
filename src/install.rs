use std::process::Command;

pub fn main(packages: Vec<String>) {
    // Install packages in test dependencies
    let _ = Command::new("elm-json")
        .arg("install")
        .arg("--test")
        .args(packages)
        .status()
        .expect("Command elm-json failed to start");
}
