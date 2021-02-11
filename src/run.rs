//! Module dealing with actually running all the tests.

use crate::make::Output;
use anyhow::Context;
use notify::{watcher, RecursiveMode, Watcher};
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::include_template;

/// Wrapper for the main_helper function with "watch" functionality.
/// This will generate, compile and run the tests.
pub fn main(
    elm_home: &Path,
    elm_project_root: &Path,
    options: crate::make::Options,
    reporter: &str,
) -> anyhow::Result<()> {
    // Prints to stderr the current version
    if !options.quiet {
        eprintln!(
            "\nelm-test-rs {} for elm 0.19.1",
            std::env!("CARGO_PKG_VERSION")
        );
        eprintln!("--------------------------------\n");
    }

    if options.watch {
        let (mut test_directories, _) =
            main_helper(&options, elm_home, elm_project_root, reporter)?;
        // Create a channel to receive the events.
        let (tx, rx) = channel();
        // Create a watcher object, delivering debounced events.
        let mut watcher = watcher(tx, Duration::from_secs(1)).context("Failed to start watcher")?;
        let recursive = RecursiveMode::Recursive;
        let elm_json_path = elm_project_root.join("elm.json");
        // Watch the elm.json and the content of source directories.
        watcher
            .watch(&elm_json_path, recursive)
            .context(format!("Failed to watch {}", elm_json_path.display()))?;
        for path in test_directories.iter() {
            watcher
                .watch(path, recursive)
                .context(format!("Failed to watch {}", path.display()))?;
        }
        loop {
            match rx.recv().context("Error watching files")? {
                notify::DebouncedEvent::NoticeWrite(_) => {}
                notify::DebouncedEvent::NoticeRemove(_) => {}
                _event => {
                    // eprintln!("{:?}", _event);
                    let (new_test_directories, _) =
                        main_helper(&options, elm_home, elm_project_root, reporter)?;
                    if new_test_directories != test_directories {
                        for path in test_directories.iter() {
                            watcher
                                .unwatch(path)
                                .context(format!("Failed to unwatch {}", path.display()))?;
                        }
                        for path in new_test_directories.iter() {
                            watcher
                                .watch(path, recursive)
                                .context(format!("Failed to watch {}", path.display()))?;
                        }
                        test_directories = new_test_directories;
                    }
                }
            }
        }
    } else {
        let (_, exit_code) = main_helper(&options, elm_home, elm_project_root, reporter)?;
        std::process::exit(exit_code);
    }
}

/// Main function, preparing and running the tests.
/// It has multiple steps that can be summarized as:
///
///  1. Generate and compile `Runner.elm` with a master test concatenating all found exposed tests.
///  2. Kernel-patch it and wrapp it into a Node worker module.
///  3. Compile `Reporter.elm` into a Node module.
///  4. Generate and start the Node supervisor program.
///
/// Returns the updated test_directories and the last exit code.
/// (useful for watch mode).
fn main_helper(
    options: &crate::make::Options,
    elm_home: &Path,
    elm_project_root: &Path,
    reporter: &str,
) -> anyhow::Result<(Vec<PathBuf>, i32)> {
    // let start_time = std::time::Instant::now();

    // Compile the Runner.elm file.
    let (test_directories, tests_root, modules_abs_paths, compiled_runner) =
        match crate::make::main_helper(options, elm_home, elm_project_root)? {
            Output::MakeFailure { test_directories } => return Ok((test_directories, 1)),
            Output::MakeSuccess {
                test_directories,
                tests_root,
                modules_abs_paths,
                compiled_runner,
            } => (
                test_directories,
                tests_root,
                modules_abs_paths,
                compiled_runner,
            ),
        };

    // Add a kernel patch to the generated code in order to be able to recognize
    // values of type Test at runtime with the `check: a -> Maybe Test` function.
    // eprintln!("Kernel-patching Runner.elm.js ...");
    let compiled_runner_src = fs::read_to_string(&compiled_runner).context(format!(
        "Failed to read newly created file {}",
        compiled_runner.display()
    ))?;
    fs::write(
        &compiled_runner,
        &kernel_patch_tests(&compiled_runner_src).context(format!(
            "Failed to patch the file {}",
            compiled_runner.display()
        ))?,
    )
    .context(format!(
        "Failed to write the patched file {}",
        compiled_runner.display()
    ))?;

    // Generate the node_runner.js node module embedding the Elm runner

    let polyfills = include_template!("node_polyfills.js");
    let node_runner_template = include_template!("node_runner.js");
    let node_runner_path = tests_root.join("js").join("node_runner.js");
    let filter = match &options.filter {
        None => "null".to_string(),
        Some(s) => format!("\"{}\"", s),
    };
    crate::make::create_templated(
        node_runner_template, // template
        &node_runner_path,    // output
        &[
            ("{{ initialSeed }}", &options.seed.to_string()),
            ("{{ fuzzRuns }}", &options.fuzz.to_string()),
            ("{{ filter }}", &filter),
            ("{{ polyfills }}", polyfills),
        ],
    )
    .context(format!("Failed to write {}", node_runner_path.display()))?;

    // Compile the Reporter.elm into Reporter.elm.js
    // eprintln!("Compiling Reporter.elm.js ...");
    let reporter_template = include_template!("Reporter.elm");
    let reporter_elm_path = tests_root.join("src").join("Reporter.elm");
    std::fs::write(&reporter_elm_path, reporter_template)
        .context("Error writing Reporter.elm to test folder")?;
    let compiled_reporter = tests_root.join("js").join("Reporter.elm.js");
    // let compile_time = std::time::Instant::now();
    if !crate::make::compile(
        elm_home,
        &tests_root,        // current_dir
        &options.compiler,  // compiler
        &compiled_reporter, // output
        &[&reporter_elm_path],
    )?
    .success()
    {
        return Ok((test_directories, 1));
    }

    // Generate the supervisor Node module
    let node_supervisor_template = include_template!("node_supervisor.js");
    let node_supervisor_js_file = tests_root.join("js").join("node_supervisor.js");
    crate::make::create_templated(
        node_supervisor_template, // template
        &node_supervisor_js_file, // output
        &[
            ("{{ workersCount }}", &options.workers.to_string()),
            ("{{ initialSeed }}", &options.seed.to_string()),
            ("{{ fuzzRuns }}", &options.fuzz.to_string()),
            ("{{ reporter }}", reporter),
            ("{{ globs }}", &serde_json::to_string(&options.files).context("Failed to convert the list of tests files passed as CLI arguments to a JSON list")?),
            ("{{ paths }}", &serde_json::to_string(&modules_abs_paths).context("Failed to convert the list of actual tests files to a JSON list")?),
            ("{{ polyfills }}", polyfills),
        ],
    )
    .context(format!(
        "Failed to write {}",
        node_supervisor_js_file.display()
    ))?;

    let node_version = Command::new("node")
        .arg("--version")
        .output()
        .context("\"node --version\" failed to start")?
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
        .context("Node supervisor failed to start")?;

    // Helper closure to write to supervisor
    let stdin = supervisor
        .stdin
        .as_mut()
        .context("Failed to open supervisor stdin")?;
    let mut writeln = |msg| -> anyhow::Result<()> {
        stdin.write_all(msg)?;
        stdin.write_all(b"\n")?;
        Ok(())
    };

    // Send runner module path to supervisor to start the work
    // eprintln!("Running tests ...");
    let node_runner_path_string = node_runner_path
        .to_str()
        .context(format!(
            "Could not convert path into a String: {}",
            node_runner_path.display()
        ))?
        .to_string();
    writeln(&node_runner_path_string.as_bytes())
        .context("Failed to write node runner path to Node supervisor stdin")?;

    // Wait for supervisor child process to end and terminate with same exit code
    let exit_code = wait_child(&mut supervisor);
    Ok((test_directories, exit_code.unwrap_or(0)))
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

/// Add a kernel patch to the generated code in order to be able to recognize
/// values of type Test at runtime with the `check: a -> Maybe Test` function.
fn kernel_patch_tests(elm_js: &str) -> anyhow::Result<String> {
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
    )?;

    let check_definition = Regex::new(
        r#"(?mx)
    ^(var\s+\$author\$project\$Runner\$check)
    \s*=\s*\$author\$project\$Runner\$checkHelperReplaceMe___;?$
"#,
    )?;

    let elm_js =
        test_variant_definition.replace_all(&elm_js, "$0 __elmTestSymbol: __elmTestSymbol,");
    let elm_js = check_definition.replace(&elm_js, "$1 = value => value && value.__elmTestSymbol === __elmTestSymbol ? $$elm$$core$$Maybe$$Just(value) : $$elm$$core$$Maybe$$Nothing;");

    Ok(["const __elmTestSymbol = Symbol('elmTestSymbol');", &elm_js].join("\n"))
}
