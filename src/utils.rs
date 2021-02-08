//! Utility functions for the other modules.

use anyhow::Context;
use std::error::Error;
use std::path::{Path, PathBuf};

#[macro_export]
#[cfg(unix)]
macro_rules! include_template {
    ($name:expr) => {
        include_str!(concat!("../templates/", $name))
    };
}

#[macro_export]
#[cfg(windows)]
macro_rules! include_template {
    ($name:expr) => {
        include_str!(concat!("..\\templates\\", $name))
    };
}

/// Find the root of the elm project (of current dir).
pub fn elm_project_root() -> anyhow::Result<PathBuf> {
    let current_dir = std::env::current_dir()
        .context("Could not retrieve the path of the current working directory")?;
    parent_traversal("elm.json", &current_dir)
        .context("I didn't find any elm.json, are you in an Elm project")
}

/// Recursively (moving up) look for the file to find.
/// Return the path of the directory containing the file or an error if not found.
pub fn parent_traversal(file_to_find: &str, current_dir: &Path) -> anyhow::Result<PathBuf> {
    if std::fs::read_dir(current_dir)
        .context("Impossible to list files in current directory")?
        .filter_map(|f| f.ok())
        .any(|f| f.file_name() == file_to_find)
    {
        Ok(current_dir.to_owned())
    } else if let Some(parent_dir) = current_dir.parent() {
        parent_traversal(file_to_find, parent_dir)
    } else {
        anyhow::bail!("File not found")
    }
}

pub fn elm_home() -> anyhow::Result<PathBuf> {
    match std::env::var_os("ELM_HOME") {
        None => default_elm_home(),
        Some(os_string) => Ok(os_string.into()),
    }
}

#[cfg(target_family = "unix")]
fn default_elm_home() -> anyhow::Result<PathBuf> {
    dirs_next::home_dir()
        .context("Unknown home directory")
        .map(|p| p.join(".elm"))
}

#[cfg(target_family = "windows")]
fn default_elm_home() -> anyhow::Result<PathBuf> {
    dirs_next::data_dir()
        .context("Unknown data directory")
        .map(|p| p.join("elm"))
}

pub fn http_fetch(url: &str) -> Result<String, Box<dyn Error>> {
    let agent = ureq::builder()
        .timeout_connect(std::time::Duration::from_secs(10))
        .build();
    let response = agent
        .get(url)
        .call()
        .context(format!("Error getting {}", url))?
        .into_string()
        .context("Error converting the http response body to a String")?;
    Ok(response)
}
