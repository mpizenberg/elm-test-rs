use crate::elm_json::Config;
use glob::glob;
use pathdiff;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

pub fn main(help: bool, version: bool, compiler: Option<String>, files: Vec<String>) {
    // The help option is prioritary over the other options
    if help {
        crate::help::main();
        return;
    // The version option is the second priority
    } else if version {
        println!("{}", std::env!("CARGO_PKG_VERSION"));
        return;
    }

    // Verify that we are in an Elm project
    let elm_project_root = crate::utils::elm_project_root().unwrap();

    // Set the compiler
    let elm_compiler = compiler.unwrap_or("elm".to_string());

    // Default with tests in the tests/ directory
    let module_globs = if files.is_empty() {
        let root_string = &elm_project_root.to_str().unwrap().to_string();
        vec![
            format!("{}/{}", root_string, "tests/*.elm"),
            format!("{}/{}", root_string, "tests/**/*.elm"),
        ]
    } else {
        files
    };

    // Get file paths of all modules in canonical form
    let module_paths: HashSet<PathBuf> = module_globs
        .iter()
        // join expanded globs for each pattern
        .flat_map(|pattern| {
            glob(pattern).expect(&format!("Failed to read glob pattern {}", pattern))
        })
        // filter out errors
        .filter_map(|x| x.ok())
        // canonical form of paths
        .map(|path| {
            path.canonicalize()
                .expect(&format!("Error in canonicalize of {:?}", path))
        })
        // collect into a set of unique values
        .collect();

    // Read project elm.json
    let elm_json_str = std::fs::read_to_string(elm_project_root.join("elm.json"))
        .expect("Unable to read elm.json");
    let info = Config::try_from(elm_json_str.as_ref()).unwrap();

    // Convert package elm.json to an application elm.json if needed
    let mut elm_json_tests = match info {
        Config::Package(package) => crate::elm_json::ApplicationConfig::try_from(&package).unwrap(),
        Config::Application(application) => application,
    };

    // Promote test dependencies to normal ones
    elm_json_tests.promote_test_dependencies();

    // Make src dirs relative to the generated tests root
    let tests_root = elm_project_root.join("elm-stuff/tests-0.19.1");
    let mut source_directories: Vec<PathBuf> = elm_json_tests
        .source_directories
        .iter()
        // Get canonical form
        .map(|path| elm_project_root.join(path).canonicalize().unwrap())
        // Get path relative to tests_root
        .map(|path| pathdiff::diff_paths(&path, &tests_root).expect("Could not get relative path"))
        .collect();

    // Add src/ and elm-test-rs/elm/src/ to the source directories
    let elm_test_rs_root = crate::utils::elm_test_rs_root().unwrap();
    source_directories.push(Path::new("src").into());
    source_directories.push(elm_test_rs_root.join("elm/src"));
    println!("source_directories:\n{:?}", source_directories);
    return;

    // Compile all test files
    todo!();
    // Find all tests
    todo!();
    // Generate the Runner.elm concatenating all tests
    todo!();
    // Compile the Reporter.elm
    todo!();
    // Generate the supervisor Node module
    todo!();
    // Start the tests supervisor
    todo!();
}
