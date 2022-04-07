use std::path::{Path, PathBuf};

/// Copy the content of the elm/ dir into
/// ~/.elm/0.19.1/packages/mpizenberg/elm-test-runner/1.0.0/
fn main() {
    // todo!();
    println!("Hello from build.rs");
    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.content_only = true;
    let installed_dir = elm_home()
        .join("0.19.1")
        .join("packages")
        .join("mpizenberg")
        .join("elm-test-runner")
        .join("5.0.0");
    let elm_stuff = Path::new("elm").join("elm-stuff");
    std::fs::remove_dir_all(&elm_stuff)
        .unwrap_or_else(|_| println!("Error removing elm/elm-stuff"));
    std::fs::remove_dir_all(&installed_dir)
        .unwrap_or_else(|_| println!("Error removing elm-test-runner package in ~/.elm/"));
    std::fs::create_dir_all(&installed_dir)
        .unwrap_or_else(|_| println!("Error creating elm-test-runner package dir in ~/.elm/"));
    fs_extra::dir::copy("elm", &installed_dir, &copy_options).unwrap_or_else(|_| {
        println!("Error copying elm-test-runner package in ~/.elm/");
        0
    });
}

pub fn elm_home() -> PathBuf {
    match std::env::var_os("ELM_HOME") {
        None => default_elm_home(),
        Some(os_string) => os_string.into(),
    }
}

#[cfg(target_family = "unix")]
fn default_elm_home() -> PathBuf {
    dirs_next::home_dir()
        .expect("Unknown home directory")
        .join(".elm")
}

#[cfg(target_family = "windows")]
fn default_elm_home() -> PathBuf {
    dirs_next::data_dir()
        .expect("Unknown data directory")
        .join("elm")
}
