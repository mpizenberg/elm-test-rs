use crate::elm_json::{Config, Dependencies};
use glob::glob;
use miniserde;
use pathdiff;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

    // Make src dirs relative to the generated tests root
    let tests_root = elm_project_root.join("elm-stuff/tests-0.19.1");
    let mut source_directories: Vec<PathBuf> = elm_json_tests
        .source_directories
        .iter()
        // Add tests/ to the list of source directories
        .chain(std::iter::once(&"tests".to_string()))
        // Get canonical form
        .map(|path| elm_project_root.join(path).canonicalize().unwrap())
        // Get path relative to tests_root
        .map(|path| pathdiff::diff_paths(&path, &tests_root).expect("Could not get relative path"))
        .collect();

    // Add src/ and elm-test-rs/elm/src/ to the source directories
    let elm_test_rs_root = crate::utils::elm_test_rs_root().unwrap();
    source_directories.push(Path::new("src").into());
    source_directories.push(elm_test_rs_root.join("elm/src"));
    elm_json_tests.source_directories = source_directories
        .iter()
        .map(|path| path.to_str().unwrap().to_string())
        .collect();

    // Promote test dependencies to normal ones
    elm_json_tests.promote_test_dependencies();

    // Write the elm.json file to disk
    let elm_json_tests_path = tests_root.join("elm.json");
    std::fs::create_dir_all(&tests_root).expect("Could not create tests dir");
    std::fs::create_dir_all(&tests_root.join("src")).expect("Could not create tests dir");
    std::fs::File::create(&elm_json_tests_path)
        .expect("Unable to create generated elm.json")
        .write_all(miniserde::json::to_string(&elm_json_tests).as_bytes())
        .expect("Unable to write to generated elm.json");

    // Finish preparing the elm.json file by solving any dependency issue (use elm-json)
    let output = Command::new("elm-json")
        .arg("solve")
        .arg("--test")
        .arg("--extra")
        .arg("elm/core")
        .arg("elm/json")
        .arg("elm/time")
        .arg("elm/random")
        .arg("--")
        .arg(&elm_json_tests_path)
        // stdio config
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .expect("command failed to start");
    let solved_dependencies: Dependencies =
        miniserde::json::from_str(std::str::from_utf8(&output.stdout).unwrap())
            .expect("Wrongly formed dependencies");
    elm_json_tests.dependencies = solved_dependencies;
    std::fs::File::create(&elm_json_tests_path)
        .expect("Unable to create generated elm.json")
        .write_all(miniserde::json::to_string(&elm_json_tests).as_bytes())
        .expect("Unable to write to generated elm.json");

    // Compile all test files
    let status = Command::new("elm")
        .arg("make")
        .arg("--output=/dev/null")
        .args(module_paths.iter())
        .current_dir(&tests_root)
        // stdio config, comment to see elm make output for debug
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .expect("Command elm make --output=/dev/null failed to start");
    if !status.success() {
        std::process::exit(1);
    }
    return;

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
