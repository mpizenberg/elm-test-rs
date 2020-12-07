//! Module dealing with actually running all the tests.

use crate::elm_json::{Config, Dependencies};
use glob::glob;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
/// Options passed as arguments.
pub struct Options {
    pub help: bool,
    pub version: bool,
    pub compiler: String,
    pub seed: u32,
    pub fuzz: u32,
    pub workers: u32,
    pub report: String,
    pub files: Vec<String>,
}

/// Main function, preparing and running the tests.
/// It has multiple steps that can be summarized as:
///
///  1. Generate the list of test modules and their file paths.
///  2. Generate a correct `elm.json` for the to-be-generated `Runner.elm`.
///  3. Compile all test files such that we know they are correct.
///  4. Find all tests.
///  5. Generate `Runner.elm` with a master test concatenating all found exposed tests.
///  6. Compile it into a JS file wrapped into a Node worker module.
///  7. Compile `Reporter.elm` into a Node module.
///  8. Generate and start the Node supervisor program.
pub fn main(options: Options) {
    // The help option is prioritary over the other options
    if options.help {
        crate::help::main();
        return;
    // The version option is the second priority
    } else if options.version {
        println!("{}", std::env!("CARGO_PKG_VERSION"));
        return;
    }

    // Verify that we are in an Elm project
    let elm_project_root = crate::utils::elm_project_root().unwrap();

    // Validate reporter
    let reporter = match options.report.as_ref() {
        "console" => "console".to_string(),
        "json" => "json".to_string(),
        "junit" => "junit".to_string(),
        value => {
            eprintln!("Wrong --report value: {}", value);
            crate::help::main();
            return;
        }
    };

    // Default with tests in the tests/ directory
    let module_globs = if options.files.is_empty() {
        let root_string = &elm_project_root.to_str().unwrap().to_string();
        vec![
            format!("{}/{}", root_string, "tests/*.elm"),
            format!("{}/{}", root_string, "tests/**/*.elm"),
        ]
    } else {
        options.files
    };

    // Get file paths of all modules in canonical form
    let module_paths: HashSet<PathBuf> = module_globs
        .iter()
        // join expanded globs for each pattern
        .flat_map(|pattern| {
            glob(pattern)
                .unwrap_or_else(|_| panic!(format!("Failed to read glob pattern {}", pattern)))
        })
        // filter out errors
        .filter_map(|x| x.ok())
        // canonical form of paths
        .map(|path| {
            path.canonicalize()
                .unwrap_or_else(|_| panic!(format!("Error in canonicalize of {:?}", path)))
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
    let tests_root = elm_project_root.join("elm-stuff").join("tests-0.19.1");
    let test_directories: Vec<PathBuf> = elm_json_tests
        .source_directories
        .iter()
        // Add tests/ to the list of source directories
        .chain(std::iter::once(&"tests".to_string()))
        // Get canonical form
        .map(|path| elm_project_root.join(path).canonicalize().unwrap())
        .collect();

    let source_directories_for_runner: Vec<PathBuf> = test_directories
        .iter()
        // Get path relative to tests_root
        .map(|path| pathdiff::diff_paths(&path, &tests_root).expect("Could not get relative path"))
        // Add src/ to the source directories
        .chain(vec!["src".into()])
        .collect();

    elm_json_tests.source_directories = source_directories_for_runner
        .iter()
        .map(|path| path.to_str().unwrap().to_string())
        .collect();

    // Promote test dependencies to normal ones
    elm_json_tests.promote_test_dependencies();

    // Write the elm.json file to disk
    let elm_json_tests_path = tests_root.join("elm.json");
    std::fs::create_dir_all(&tests_root.join("src")).expect("Could not create tests dir");
    std::fs::File::create(&elm_json_tests_path)
        .expect("Unable to create generated elm.json")
        .write_all(miniserde::json::to_string(&elm_json_tests).as_bytes())
        .expect("Unable to write to generated elm.json");

    // Finish preparing the elm.json file by solving any dependency issue (use elm-json)
    eprintln!("Running elm-json to solve dependency issues ...");
    let output = Command::new("elm-json")
        .arg("solve")
        .arg("--test")
        .arg("--extra")
        .arg("elm/core")
        .arg("elm/json")
        .arg("elm/time")
        .arg("elm/random")
        .arg("billstclair/elm-xml-eeue56")
        .arg("jorgengranseth/elm-string-format")
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
    // Add a placeholder package to the dependencies.
    // The build.rs step of compilation has replaced it by our package in elm/
    elm_json_tests.dependencies.direct.insert(
        "mpizenberg/elm-placeholder-pkg".to_string(),
        "1.0.0".to_string(),
    );
    std::fs::File::create(&elm_json_tests_path)
        .expect("Unable to create generated elm.json")
        .write_all(miniserde::json::to_string(&elm_json_tests).as_bytes())
        .expect("Unable to write to generated elm.json");

    // Compile all test files
    eprintln!("Compiling all test files ...");
    compile(
        &tests_root,       // current_dir
        &options.compiler, // compiler
        "/dev/null",       // output
        &module_paths,     // src
    );

    // Find all modules and tests
    eprintln!("Finding all modules and tests ...");
    let all_modules_and_tests = crate::elmi::all_tests(&tests_root, &module_paths).unwrap();
    let runner_imports: Vec<String> = all_modules_and_tests
        .iter()
        .map(|m| "import ".to_string() + &m.module_name)
        .collect();
    let runner_tests: Vec<String> = all_modules_and_tests
        .iter()
        .map(|module| {
            let full_module_tests: Vec<String> = module
                .tests
                .iter()
                .map(|test| format!("{}.{}", &module.module_name, test))
                .collect();
            format!(
                r#"Test.describe "{}" [ {} ]"#,
                &module.module_name,
                full_module_tests.join(", ")
            )
        })
        .collect();

    // Generate templated src/Runner.elm
    #[cfg(unix)]
    let runner_template = include_str!("../templates/Runner.elm");
    #[cfg(windows)]
    let runner_template = include_str!("..\\templates\\Runner.elm");
    create_templated(
        runner_template,                           // template
        tests_root.join("src").join("Runner.elm"), // output
        &[
            ("{{ user_imports }}", &runner_imports.join("\n")),
            ("{{ tests }}", &runner_tests.join("\n    , ")),
        ],
    );

    // Compile the src/Runner.elm file into Runner.elm.js
    eprintln!("Compiling the generated templated src/Runner.elm ...");
    let compiled_runner = tests_root.join("js").join("Runner.elm.js");
    compile(
        &tests_root,       // current_dir
        &options.compiler, // compiler
        &compiled_runner,  // output
        &[Path::new("src").join("Runner.elm")],
    );

    // Generate the node_runner.js node module embedding the Elm runner
    #[cfg(unix)]
    let polyfills = include_str!("../templates/node_polyfills.js");
    #[cfg(windows)]
    let polyfills = include_str!("..\\templates\\node_polyfills.js");
    #[cfg(unix)]
    let node_runner_template = include_str!("../templates/node_runner.js");
    #[cfg(windows)]
    let node_runner_template = include_str!("..\\templates\\node_runner.js");
    let node_runner_path = tests_root.join("js").join("node_runner.js");
    create_templated(
        node_runner_template, // template
        &node_runner_path,    // output
        &[
            ("{{ initialSeed }}", &options.seed.to_string()),
            ("{{ fuzzRuns }}", &options.fuzz.to_string()),
            ("{{ polyfills }}", polyfills),
        ],
    );

    // Compile the Reporter.elm into Reporter.elm.js
    eprintln!("Compiling Reporter.elm.js ...");
    #[cfg(unix)]
    let reporter_template = include_str!("../templates/Reporter.elm");
    #[cfg(windows)]
    let reporter_template = include_str!("..\\templates\\Reporter.elm");
    let reporter_elm_path = tests_root.join("src").join("Reporter.elm");
    std::fs::write(&reporter_elm_path, reporter_template)
        .expect("Error writing Reporter.elm to test folder");
    let compiled_reporter = tests_root.join("js").join("Reporter.elm.js");
    compile(
        &tests_root,        // current_dir
        &options.compiler,  // compiler
        &compiled_reporter, // output
        &[&reporter_elm_path],
    );

    // Generate the supervisor Node module
    #[cfg(unix)]
    let node_supervisor_template = include_str!("../templates/node_supervisor.js");
    #[cfg(windows)]
    let node_supervisor_template = include_str!("..\\templates\\node_supervisor.js");
    create_templated(
        node_supervisor_template,                         // template
        tests_root.join("js").join("node_supervisor.js"), // output
        &[
            ("{{ nb_workers }}", &options.workers.to_string()),
            ("{{ initialSeed }}", &options.seed.to_string()),
            ("{{ fuzzRuns }}", &options.fuzz.to_string()),
            ("{{ reporter }}", &reporter),
            ("{{ polyfills }}", polyfills),
        ],
    );

    // Start the tests supervisor
    eprintln!("Starting the supervisor ...");
    let mut supervisor = Command::new("node")
        .arg("js/node_supervisor.js")
        .current_dir(&tests_root)
        .stdin(Stdio::piped())
        .spawn()
        .expect("command failed to start");

    // Helper closure to write to supervisor
    let stdin = supervisor.stdin.as_mut().expect("Failed to open stdin");
    let mut writeln = |msg| {
        stdin.write_all(msg).expect("writeln");
        stdin.write_all(b"\n").expect("writeln");
    };

    // Send runner module path to supervisor to start the work
    eprintln!("Running tests ...");
    let node_runner_path_string = node_runner_path.to_str().unwrap().to_string();
    writeln(&node_runner_path_string.as_bytes());

    // Wait for supervisor child process to end and terminate with same exit code
    let exit_code = wait_child(&mut supervisor);
    eprintln!("Exited with code {:?}", exit_code);
    std::process::exit(exit_code.unwrap_or(1));
}

/// Wait for child process to end
fn wait_child(child: &mut std::process::Child) -> Option<i32> {
    match child.try_wait() {
        Ok(Some(status)) => status.code(),
        Ok(None) => match child.wait() {
            Ok(status) => status.code(),
            _ => None,
        },
        Err(e) => {
            eprintln!("Error attempting to wait for child: {}", e);
            None
        }
    }
}

/// Compile an Elm module into a JS file (without --optimized)
fn compile<P1, P2, I, S>(current_dir: P1, compiler: &str, output: P2, src: I)
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(compiler)
        .arg("make")
        .arg(format!("--output={}", output.as_ref().to_str().unwrap()))
        .args(src)
        .current_dir(current_dir)
        // stdio config, comment to see elm make output for debug
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .expect("Command elm make failed to start");
    if !status.success() {
        std::process::exit(1);
    }
}

/// Replace the template keys and write result to output file.
fn create_templated<P: AsRef<Path>>(template: &str, output: P, replacements: &[(&str, &str)]) {
    let mut output_str = template.to_string();
    replacements
        .iter()
        .for_each(|(from, to)| output_str = output_str.replacen(from, to, 1));
    std::fs::write(output, output_str).expect("Unable to write to generated file");
}
