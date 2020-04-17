//! Initialize elm tests.

use std::path::Path;
use std::process::Command;

/// Use the elm-json binary tool to copy the behavior of the command `elm-test init`.
pub fn main() {
    // Install elm-explorations/test
    let status = Command::new("elm-json")
        .arg("install")
        .arg("--test")
        .arg("elm-explorations/test@1")
        .status()
        .expect("Command elm-json failed to start");
    if !status.success() {
        eprintln!(
            "There was an error when trying to add elm-explorations/test to your dependencies"
        );
        std::process::exit(1);
    }

    // Create the tests/Tests.elm template
    std::fs::create_dir_all("tests").expect("Impossible to create directory tests/");
    let template_path = crate::utils::elm_test_rs_root()
        .unwrap()
        .join("templates/Tests.elm");
    if !Path::new("tests/Tests.elm").exists() {
        std::fs::copy(template_path, "tests/Tests.elm").expect("Unable to copy Tests.elm template");
    }
}
