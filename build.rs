use fs_extra;

/// Copy the content of the elm/ dir into
/// ~/.elm/0.19.1/packages/mpizenberg/elm-placeholder-pkg/1.0.0/
fn main() {
    // todo!();
    println!("Hello from build.rs");
    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.content_only = true;
    let installed_dir = "/home/matthieu/.elm/0.19.1/packages/mpizenberg/elm-placeholder-pkg/1.0.0";
    std::fs::remove_dir_all("elm/elm-stuff")
        .unwrap_or_else(|_| println!("Error removing elm/elm-stuff"));
    std::fs::remove_dir_all(installed_dir)
        .unwrap_or_else(|_| println!("Error removing elm-test-runner package in ~/.elm/"));
    std::fs::create_dir_all(installed_dir)
        .unwrap_or_else(|_| println!("Error creating elm-test-runner package dir in ~/.elm/"));
    fs_extra::dir::copy("elm", installed_dir, &copy_options).unwrap_or_else(|_| {
        println!("Error copying elm-test-runner package in ~/.elm/");
        0
    });
}
