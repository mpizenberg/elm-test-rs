use crate::elm_json::Config;
use glob::glob;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::PathBuf;

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
    println!("elm_json_tests:\n{:?}", elm_json_tests);
    return;
    let elm_test_rs_root = crate::utils::elm_test_rs_root().unwrap();
    let elm_test_rs_src_dirs = elm_test_rs_root.join("elm/src");

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
