//! Module dealing with actually running all the tests.

use glob::glob;
use notify::{watcher, RecursiveMode, Watcher};
use pubgrub_dependency_provider_elm::project_config::ProjectConfig;
use regex::Regex;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug)]
/// Options passed as arguments.
pub struct Options {
    pub help: bool,
    pub version: bool,
    pub watch: bool,
    pub compiler: String,
    pub seed: u32,
    pub fuzz: u32,
    pub workers: u32,
    pub filter: Option<String>,
    pub report: String,
    pub connectivity: crate::deps::ConnectivityStrategy,
    pub files: Vec<String>,
}

/// Main function, preparing and running the tests.
/// It has multiple steps that can be summarized as:
///
///  1. Generate the list of test modules and their file paths.
///  2. Generate a correct `elm.json` for the to-be-generated `Runner.elm`.
///  3. Find all tests.
///  4. Generate `Runner.elm` with a master test concatenating all found exposed tests.
///  5. Compile it into a JS file wrapped into a Node worker module.
///  6. Compile `Reporter.elm` into a Node module.
///  7. Generate and start the Node supervisor program.
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

    // Prints to stderr the current version
    eprintln!("\nelm-test-rs {}", std::env!("CARGO_PKG_VERSION"));
    eprintln!("-----------------\n");

    // Verify that we are in an Elm project
    let elm_project_root = crate::utils::elm_project_root().unwrap();

    // Validate reporter mode
    let reporter = match options.report.as_ref() {
        "console" => console_color_mode().to_string(), // returns "consoleColor" or "consoleNoColor"
        "consoleDebug" => "consoleDebug".to_string(),
        "consoleColor" => "consoleColor".to_string(),
        "consoleNoColor" => "consoleNoColor".to_string(),
        "json" => "json".to_string(),
        "junit" => "junit".to_string(),
        "exercism" => "exercism".to_string(),
        value => {
            eprintln!("Wrong --report value: {}", value);
            crate::help::main();
            std::process::exit(1);
        }
    };

    if options.watch {
        let mut test_directories = main_helper(&options, &elm_project_root, &reporter);
        // Create a channel to receive the events.
        let (tx, rx) = channel();
        // Create a watcher object, delivering debounced events.
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        let recursive = RecursiveMode::Recursive;
        let elm_json_path = elm_project_root.join("elm.json");
        // Watch the elm.json and the content of source directories.
        watcher.watch(elm_json_path, recursive).unwrap();
        for path in test_directories.iter() {
            watcher.watch(path, recursive).unwrap();
        }
        loop {
            match rx.recv() {
                Ok(notify::DebouncedEvent::NoticeWrite(_)) => {}
                Ok(notify::DebouncedEvent::NoticeRemove(_)) => {}
                Ok(_event) => {
                    // eprintln!("{:?}", _event);
                    let new_test_directories = main_helper(&options, &elm_project_root, &reporter);
                    if new_test_directories != test_directories {
                        for path in test_directories.iter() {
                            watcher.unwatch(path).unwrap();
                        }
                        for path in new_test_directories.iter() {
                            watcher.watch(path, recursive).unwrap();
                        }
                        test_directories = new_test_directories;
                    }
                }
                Err(e) => eprintln!("watch error: {:?}", e),
            }
        }
    } else {
        main_helper(&options, &elm_project_root, &reporter);
    }
}

/// Returns "consoleColor" or "consoleNoColor" based on the following two standards:
///  - https://bixense.com/clicolors/
///  - https://no-color.org/
fn console_color_mode() -> &'static str {
    if &std::env::var("CLICOLOR_FORCE").unwrap_or_else(|_| "0".to_string()) != "0" {
        "consoleColor"
    } else if std::env::var("NO_COLOR").is_ok() {
        "consoleNoColor"
    } else {
        match (
            atty::is(atty::Stream::Stdout),
            std::env::var("CLICOLOR").as_deref(),
        ) {
            (false, _) => "consoleNoColor",
            (true, Ok("0")) => "consoleNoColor",
            (true, _) => "consoleColor",
        }
    }
}

fn main_helper(options: &Options, elm_project_root: &Path, reporter: &str) -> Vec<PathBuf> {
    let start_time = std::time::Instant::now();
    // Default with tests in the tests/ directory
    let module_globs = if options.files.is_empty() {
        let root_string = elm_project_root.to_str().unwrap().to_string();
        vec![
            format!("{}/{}", root_string, "tests/*.elm"),
            format!("{}/{}", root_string, "tests/**/*.elm"),
        ]
    } else {
        options.files.clone()
    };

    // Get file paths of all modules in canonical form (absolute path)
    let modules_abs_paths: HashSet<PathBuf> = module_globs
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
    // let info = Config::try_from(elm_json_str.as_ref()).unwrap();
    let info: ProjectConfig = serde_json::from_str(&elm_json_str).unwrap();
    let source_directories = match &info {
        ProjectConfig::Application(app_config) => app_config.source_directories.clone(),
        ProjectConfig::Package(_) => vec!["src".to_string()],
    };

    // Make src dirs relative to the generated tests root
    let tests_root = elm_project_root.join("elm-stuff").join("tests-0.19.1");
    let mut test_directories: Vec<PathBuf> = source_directories
        .iter()
        // Get canonical paths
        .map(|path| elm_project_root.join(path).canonicalize().unwrap())
        .collect();
    // Add tests/ to the list of source directories
    if let Ok(path) = elm_project_root.join("tests").canonicalize() {
        test_directories.push(path);
    }
    let source_directories_for_runner: Vec<PathBuf> = test_directories
        .iter()
        // Get path relative to tests_root
        .map(|path| pathdiff::diff_paths(&path, &tests_root).expect("Could not get relative path"))
        // Add src/ to the source directories
        .chain(vec!["src".into()])
        .collect();

    // Generate an elm.json for the to-be-generated Runner.elm.
    // eprintln!("Generating the elm.json for the Runner.elm");
    let tests_config =
        crate::deps::solve(&options.connectivity, &info, &source_directories_for_runner).unwrap();
    match options.connectivity {
        crate::deps::ConnectivityStrategy::Progressive => (),
        _ => eprintln!(
            "The dependencies picked for your chosen connectivity are:\n{}",
            serde_json::to_string_pretty(&tests_config.dependencies).unwrap(),
        ),
    };
    let tests_config = ProjectConfig::Application(tests_config);
    let tests_config_path = tests_root.join("elm.json");
    std::fs::create_dir_all(tests_root.join("src")).expect("Could not create tests dir");
    let tests_config_str = serde_json::to_string(&tests_config).unwrap();
    match std::fs::read_to_string(&tests_config_path) {
        Ok(old_conf) if tests_config_str == old_conf => (),
        _ => std::fs::write(tests_config_path, tests_config_str)
            .expect("Unable to write to generated elm.json"),
    };

    // Find module names
    let module_names: Vec<String> = modules_abs_paths
        .iter()
        .map(|p| get_module_name(&test_directories, p))
        .collect();

    // Runner.elm imports of tests modules
    let imports: Vec<String> = module_names
        .iter()
        .map(|m| format!("import {}", m))
        .collect();

    // Find all potential tests
    // eprintln!("Finding all potential tests ...");
    let potential_tests: Vec<String> = module_names
        .iter()
        .zip(modules_abs_paths)
        .map(|(module_name, path)| {
            let source = fs::read_to_string(path).unwrap();
            crate::parser::potential_tests(&source)
                .iter()
                .map(move |potential_test| format!("check {}.{}", module_name, potential_test))
                .collect::<Vec<_>>()
        })
        .flatten()
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
            ("{{ imports }}", &imports.join("\n")),
            ("{{ potential_tests }}", &potential_tests.join("\n    , ")),
        ],
    );

    // Compile the src/Runner.elm file into Runner.elm.js
    let _preparation_time = start_time.elapsed().as_secs_f32();
    // eprintln!("Spent {}s generating Runner.elm", _preparation_time);
    // eprintln!("Compiling the generated templated src/Runner.elm ...");
    let compiled_runner = tests_root.join("js").join("Runner.elm.js");
    let compile_time = std::time::Instant::now();
    if !compile(
        &tests_root,       // current_dir
        &options.compiler, // compiler
        &compiled_runner,  // output
        &[Path::new("src").join("Runner.elm")],
    ) {
        if options.watch {
            return test_directories;
        } else {
            std::process::exit(1);
        }
    }
    let mut total_time_elm_compiler = compile_time.elapsed().as_secs_f32();

    // Add a kernel patch to the generated code in order to be able to recognize
    // values of type Test at runtime with the `check: a -> Maybe Test` function.
    // eprintln!("Kernel-patching Runner.elm.js ...");
    let compiled_runner_src =
        fs::read_to_string(&compiled_runner).expect("Cannot read newly created elm.js file");
    fs::write(&compiled_runner, &kernel_patch_tests(&compiled_runner_src))
        .expect("Cannot write updated elm.js file");

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
    let filter = match &options.filter {
        None => "null".to_string(),
        Some(s) => format!("\"{}\"", s),
    };
    create_templated(
        node_runner_template, // template
        &node_runner_path,    // output
        &[
            ("{{ initialSeed }}", &options.seed.to_string()),
            ("{{ fuzzRuns }}", &options.fuzz.to_string()),
            ("{{ filter }}", &filter),
            ("{{ polyfills }}", polyfills),
        ],
    );

    // Compile the Reporter.elm into Reporter.elm.js
    // eprintln!("Compiling Reporter.elm.js ...");
    #[cfg(unix)]
    let reporter_template = include_str!("../templates/Reporter.elm");
    #[cfg(windows)]
    let reporter_template = include_str!("..\\templates\\Reporter.elm");
    let reporter_elm_path = tests_root.join("src").join("Reporter.elm");
    std::fs::write(&reporter_elm_path, reporter_template)
        .expect("Error writing Reporter.elm to test folder");
    let compiled_reporter = tests_root.join("js").join("Reporter.elm.js");
    let compile_time = std::time::Instant::now();
    if !compile(
        &tests_root,        // current_dir
        &options.compiler,  // compiler
        &compiled_reporter, // output
        &[&reporter_elm_path],
    ) {
        if options.watch {
            return test_directories;
        } else {
            std::process::exit(1);
        }
    }
    total_time_elm_compiler += compile_time.elapsed().as_secs_f32();
    eprintln!(
        "Total time spent in the elm compiler: {}s",
        total_time_elm_compiler
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
            ("{{ workersCount }}", &options.workers.to_string()),
            ("{{ initialSeed }}", &options.seed.to_string()),
            ("{{ fuzzRuns }}", &options.fuzz.to_string()),
            ("{{ reporter }}", &reporter),
            ("{{ polyfills }}", polyfills),
        ],
    );

    let node_version = Command::new("node")
        .arg("--version")
        .output()
        .expect("command failed to start")
        .stdout;

    // Node supports worker_threads as experimental feature since 10.5,
    // but it is unknown whether all versions since 10.5 actually work with elm-test-rs.
    let mut supervisor_args = Vec::new();
    if node_version.starts_with(b"v10.") {
        supervisor_args.push("--experimental-worker");
    }
    supervisor_args.push("js/node_supervisor.js");

    // Start the tests supervisor
    // eprintln!("Starting the supervisor ...");
    let mut supervisor = Command::new("node")
        .args(supervisor_args)
        .current_dir(tests_root)
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
    // eprintln!("Running tests ...");
    let node_runner_path_string = node_runner_path.to_str().unwrap().to_string();
    writeln(&node_runner_path_string.as_bytes());

    // Wait for supervisor child process to end and terminate with same exit code
    let exit_code = wait_child(&mut supervisor);
    // eprintln!("Exited with code {:?}", exit_code);
    if options.watch {
        test_directories
    } else {
        std::process::exit(exit_code.unwrap_or(0));
    }
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
fn compile<P1, P2, I, S>(current_dir: P1, compiler: &str, output: P2, src: I) -> bool
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
    status.success()
}

/// Replace the template keys and write result to output file.
fn create_templated<P: AsRef<Path>>(template: &str, output: P, replacements: &[(&str, &str)]) {
    let mut output_str = template.to_string();
    replacements
        .iter()
        .for_each(|(from, to)| output_str = output_str.replacen(from, to, 1));
    std::fs::write(output, output_str).expect("Unable to write to generated file");
}

// By finding the module name from the file path we can import it even if
// the file is full of errors. Elm will then report what’s wrong.
fn get_module_name(
    source_dirs: impl IntoIterator<Item = impl AsRef<Path>>,
    file: impl AsRef<Path>,
) -> String {
    // eprintln!("get_module_name of: {}", file.as_ref().display());
    let file = file.as_ref();
    let matching_source_dir = {
        let mut matching = source_dirs.into_iter().filter(|dir| file.starts_with(dir));
        match (matching.next(), matching.next()) {
            (None, None) => {
                panic!(
                    "This file:
{}
...matches no source directory! Imports won’t work then.
",
                    file.display()
                )
            }
            (Some(dir), None) => dir,
            _ => panic!("2+ matching source dirs"),
        }
    };

    let trimmed: PathBuf = file
        .strip_prefix(matching_source_dir)
        .unwrap()
        .with_extension("");
    let module_name_parts: Vec<_> = trimmed.iter().map(|s| s.to_str().unwrap()).collect();
    module_name_parts
        .iter()
        .filter(|s| !is_valid_module_name(s))
        .for_each(|s| eprintln!("{}", s));
    assert!(module_name_parts.iter().all(|s| is_valid_module_name(s)));
    assert!(!module_name_parts.is_empty());
    module_name_parts.join(".")
}

fn is_valid_module_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().next().unwrap().is_uppercase()
        && name.chars().all(|c| c == '_' || c.is_alphanumeric())
}

/// Add a kernel patch to the generated code in order to be able to recognize
/// values of type Test at runtime with the `check: a -> Maybe Test` function.
fn kernel_patch_tests(elm_js: &str) -> String {
    // For older versions of elm-explorations/test we need to list every single
    // variant of the `Test` type. To avoid having to update this regex if a new
    // variant is added, newer versions of elm-explorations/test have prefixed all
    // variants with `ElmTestVariant__` so we can match just on that.
    let test_variant_definition = Regex::new(
        r#"(?mx)
    ^var\s+\$elm_explorations\$test\$Test\$Internal\$
    (?:ElmTestVariant__\w+|UnitTest|FuzzTest|Labeled|Skipped|Only|Batch)
    \s*=\s*(?:\w+\(\s*)?function\s*\([\w,\s]*\)\s*\{\s*return\s*\{
"#,
    )
    .unwrap();

    let check_definition = Regex::new(
        r#"(?mx)
    ^(var\s+\$author\$project\$Runner\$check)
    \s*=\s*\$author\$project\$Runner\$checkHelperReplaceMe___;?$
"#,
    )
    .unwrap();

    let elm_js =
        test_variant_definition.replace_all(&elm_js, "$0 __elmTestSymbol: __elmTestSymbol,");
    let elm_js = check_definition.replace(&elm_js, "$1 = value => value && value.__elmTestSymbol === __elmTestSymbol ? $$elm$$core$$Maybe$$Just(value) : $$elm$$core$$Maybe$$Nothing;");

    ["const __elmTestSymbol = Symbol('elmTestSymbol');", &elm_js].join("\n")
}
