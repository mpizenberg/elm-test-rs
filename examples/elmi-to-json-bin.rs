#![allow(non_snake_case)]

use miniserde::{json, Deserialize};
use std::path::Path;
use std::process::{Command, Stdio};

/// usage: cargo run --example elmi-to-json-bin -- /path/to/elm/project
pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let elm_project = &args[1];
    let work_dir = Path::new(elm_project)
        .join("elm-stuff/generated-code/elm-community/elm-test/0.19.1-revision2")
        .canonicalize()
        .expect("Woops, wrong work_dir path");
    let output = Command::new("elmi-to-json")
        .arg("--for-elm-test")
        .arg("--elm-version")
        .arg("0.19.1")
        // stdio config
        .current_dir(&work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .expect("command failed to start");
    let str_output = std::str::from_utf8(&output.stdout).expect("utf8");
    let output: ElmiToJsonOutput = json::from_str(str_output).expect("deserialize");
    // output.testModules.iter().filter_map(|entry| entry.tests non empty & test_file_paths contains canonicalized entry.path)
    // No need to verify that module names are valid, elm 0.19.1 already verifies that.
    // No need to filter exposed since only exposed values are in elmi files.
    let modules = output.testModules;
    println!("{:?}", &modules);
}

#[derive(Deserialize, Debug)]
struct ElmiToJsonOutput {
    testModules: Vec<TestModule>,
}

#[derive(Deserialize, Debug)]
struct TestModule {
    moduleName: String,
    path: String,
    tests: Vec<String>,
}
