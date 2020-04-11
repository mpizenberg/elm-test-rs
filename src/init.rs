use std::path::{Path, PathBuf};
use std::process::Command;

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
    let current_exe_path = std::env::current_exe().expect("Impossible to get elm-test-rs path");
    let mut template_path = parent_traversal("Cargo.toml", current_exe_path.parent().unwrap())
        .expect("There was an error while searching for the template Tests.elm file");
    template_path.push("templates/Tests.elm");
    std::fs::copy(template_path, "tests/Tests.elm").expect("Unable to copy Tests.elm template");
}

/// Recursively (moving up) look for the file to find
fn parent_traversal(file_to_find: &str, current_dir: &Path) -> std::io::Result<PathBuf> {
    if std::fs::read_dir(current_dir)?.any(|f| f.unwrap().file_name() == file_to_find) {
        Ok(current_dir.to_owned())
    } else if let Some(parent_dir) = current_dir.parent() {
        parent_traversal(file_to_find, parent_dir)
    } else {
        Ok(Path::new("/").to_owned())
    }
}
