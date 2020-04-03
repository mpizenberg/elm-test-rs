use std::path::{Path, PathBuf};

/// cargo run --example find_root -- "Cargo.toml"
pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_to_find = &args[1];
    let current_exe = std::env::current_exe().unwrap();
    let current_dir = current_exe.parent().unwrap().to_owned();
    let dir = parent_traversal(&file_to_find, &current_dir);
    println!("root: {:?}", dir);
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
