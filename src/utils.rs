//! Utility functions for the other modules.

use std::error::Error;
use std::path::{Path, PathBuf};

/// Find the root of the elm project (of current dir).
pub fn elm_project_root() -> Result<PathBuf, Box<dyn Error>> {
    let current_dir = std::env::current_dir()?;
    parent_traversal("elm.json", &current_dir)
        .map_err(|_| "I didn't find any elm.json, are you in an Elm project".into())
}

/// Recursively (moving up) look for the file to find.
/// Return the path of the directory containing the file or an error if not found.
pub fn parent_traversal(file_to_find: &str, current_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if std::fs::read_dir(current_dir)?.any(|f| f.unwrap().file_name() == file_to_find) {
        Ok(current_dir.to_owned())
    } else if let Some(parent_dir) = current_dir.parent() {
        parent_traversal(file_to_find, parent_dir)
    } else {
        Err("File not found".into())
    }
}

pub fn elm_home() -> PathBuf {
    match std::env::var_os("ELM_HOME") {
        None => default_elm_home(),
        Some(os_string) => os_string.into(),
    }
}

#[cfg(target_family = "unix")]
fn default_elm_home() -> PathBuf {
    dirs::home_dir()
        .expect("Unknown home directory")
        .join(".elm")
}

#[cfg(target_family = "windows")]
fn default_elm_home() -> PathBuf {
    dirs::data_dir()
        .expect("Unknown data directory")
        .join("elm")
}

pub fn http_fetch(url: &str) -> Result<String, Box<dyn Error>> {
    ureq::get(url)
        .timeout_connect(10_000)
        .call()
        .into_string()
        .map_err(|e| e.into())
}
