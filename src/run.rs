use crate::elm_json::{Config, Dependencies};
use glob::glob;
use miniserde;
use pathdiff;
use rand::Rng;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct Options {
    pub help: bool,
    pub version: bool,
    pub compiler: Option<String>,
    pub seed: Option<u32>,
    pub fuzz: Option<u32>,
    pub report: Option<String>,
    pub files: Vec<String>,
}

// pub fn main(help: bool, version: bool, compiler: Option<String>, files: Vec<String>) {
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

    // Set the compiler
    let elm_compiler = options.compiler.unwrap_or("elm".to_string());

    // Default seed and fuzz are random if not provided
    let mut rng = rand::thread_rng();
    let initial_seed: u32 = options.seed.unwrap_or(rng.gen());
    let fuzz_runs: u32 = options.fuzz.unwrap_or(rng.gen());

    // Default reporter is console if not provided
    let reporter = match options.report.as_deref() {
        None => "console".to_string(),
        Some("console") => "console".to_string(),
        Some("json") => "json".to_string(),
        Some("junit") => "junit".to_string(),
        Some(value) => {
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
        .arg("billstclair/elm-xml-eeue56")
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
    compile(
        &tests_root,                        // current_dir
        &elm_compiler,                      // compiler
        &Path::new("/dev/null").to_owned(), // output
        module_paths.iter(),                // src
    );

    // Find all modules and tests
    let all_modules_and_tests = crate::elmi::all_tests(&tests_root, &module_paths).unwrap();
    let runner_imports: Vec<String> = all_modules_and_tests
        .iter()
        .map(|m| "import ".to_string() + &m.module_name)
        .collect();
    let runner_tests: Vec<String> = all_modules_and_tests
        .iter()
        .flat_map(|m| {
            m.tests
                .iter()
                .map(move |test| m.module_name.clone() + "." + test)
        })
        .collect();

    // Generate templated src/Runner.elm
    create_templated(
        elm_test_rs_root.join("templates/Runner.elm"), // template
        tests_root.join("src/Runner.elm"),             // output
        vec![
            ("user_imports".to_string(), runner_imports.join("\n")),
            ("tests".to_string(), runner_tests.join(", ")),
        ],
    );

    // Compile the src/Runner.elm file into Runner.elm.js
    let compiled_elm_file = tests_root.join("Runner.elm.js");
    compile(
        &tests_root,         // current_dir
        &elm_compiler,       // compiler
        &compiled_elm_file,  // output
        &["src/Runner.elm"], // src
    );

    // Generate the node_runner.js node module embedding the Elm runner
    let polyfills = std::fs::read_to_string(&elm_test_rs_root.join("templates/polyfills.js"))
        .expect("polyfills.js template missing");
    let compiled_elm =
        std::fs::read_to_string(&compiled_elm_file).expect("Compiled Elm runner file missing");
    create_templated(
        elm_test_rs_root.join("templates/runner.js"), // template
        tests_root.join("node_runner.js"),            // output
        vec![
            ("polyfills".to_string(), polyfills),
            ("compiled_elm".to_string(), compiled_elm),
            ("initialSeed".to_string(), initial_seed.to_string()),
            ("fuzzRuns".to_string(), fuzz_runs.to_string()),
        ],
    );

    // Compile the Reporter.elm into Reporter.elm.js
    let compiled_reporter = tests_root.join("Reporter.elm.js");
    compile(
        &tests_root,        // current_dir
        &elm_compiler,      // compiler
        &compiled_reporter, // output
        &[elm_test_rs_root.join("elm/src/ElmTestRs/Test/Reporter.elm")],
    );

    // Generate the node_reporter.js module embedding the Elm reporter
    let compiled_elm =
        std::fs::read_to_string(&compiled_reporter).expect("Compiled Elm reporter file missing");
    create_templated(
        elm_test_rs_root.join("templates/reporter.js"), // template
        tests_root.join("node_reporter.js"),            // output
        vec![
            ("compiled_elm".to_string(), compiled_elm),
            ("initialSeed".to_string(), initial_seed.to_string()),
            ("fuzzRuns".to_string(), fuzz_runs.to_string()),
            ("reporter".to_string(), reporter.clone()),
            ("nbTests".to_string(), runner_tests.len().to_string()),
        ],
    );
    return;

    // Generate the supervisor Node module
    todo!();
    // Start the tests supervisor
    todo!();
}

/// Compile an Elm module into a JS file.
fn compile<P, I, S>(current_dir: P, compiler: &str, output: P, src: I)
where
    P: AsRef<Path>,
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
fn create_templated<P: AsRef<Path>>(template: P, output: P, replacements: Vec<(String, String)>) {
    let template_content = std::fs::read_to_string(template).expect("template missing");
    let content = varj::parse(&template_content, &replacements.into_iter().collect())
        .expect("The template does not match with the replacement keys");
    std::fs::File::create(output)
        .expect("Unable to create generated file")
        .write_all(content.as_bytes())
        .expect("Unable to write to generated file");
}
