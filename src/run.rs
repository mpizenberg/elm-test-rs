//! Module dealing with actually running all the tests.

use crate::make::Output;
use crate::project::Project;
use anyhow::Context;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::num::NonZeroU32;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::include_template;

#[derive(Debug)]
/// Options passed as arguments.
pub struct Options {
    pub seed: u32,
    pub fuzz: NonZeroU32,
    pub workers: u32,
    pub filter: Option<String>,
    pub reporter: String,
}

/// Wrapper for the main_helper function with "watch" functionality.
/// This will generate, compile and run the tests.
///
/// TODO: For the time being, this returns the error code, but we should improve this when
/// `std::process::Termination` lands in stable.
pub fn main(
    elm_home: &Path,
    elm_project_root: &Path,
    make_options: crate::make::Options,
    run_options: Options,
) -> anyhow::Result<i32> {
    // Prints to stderr the current version
    let title = format!(
        "elm-test-rs {} for elm 0.19.1",
        std::env!("CARGO_PKG_VERSION")
    );
    log::warn!("\n{}\n{}\n", &title, "-".repeat(title.len()));

    let mut project = Project::from_dir(elm_project_root.to_path_buf())?;
    if make_options.watch {
        project.watch(|project| {
            main_helper(elm_home, project, &make_options, &run_options).map(|_| ())
        })?;
        Ok(0)
    } else {
        main_helper(elm_home, &project, &make_options, &run_options)
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
/// Returns the the last exit code.
fn main_helper(
    elm_home: &Path,
    project: &Project,
    make_options: &crate::make::Options,
    run_options: &Options,
) -> anyhow::Result<i32> {
    // let start_time = std::time::Instant::now();

    // Compile the Runner.elm file.
    let (tests_root, modules_abs_paths, compiled_runner) =
        match crate::make::main_helper(elm_home, project, make_options)? {
            Output::MakeFailure => return Ok(1),
            Output::MakeSuccess {
                tests_root,
                modules_abs_paths,
                compiled_runner,
            } => (tests_root, modules_abs_paths, compiled_runner),
        };

    // Add a kernel patch to the generated code in order to be able to recognize
    // values of type Test at runtime with the `check: a -> Maybe Test` function.
    log::info!("Kernel-patching Runner.elm.js ...");
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
    let filter = match &run_options.filter {
        None => "null".to_string(),
        Some(s) => format!("\"{}\"", s),
    };
    crate::make::create_templated(
        node_runner_template, // template
        &node_runner_path,    // output
        &[
            ("{{ initialSeed }}", &run_options.seed.to_string()),
            ("{{ fuzzRuns }}", &run_options.fuzz.to_string()),
            ("{{ filter }}", &filter),
            ("{{ polyfills }}", polyfills),
        ],
    )
    .context(format!("Failed to write {}", node_runner_path.display()))?;

    // Compile the Reporter.elm into Reporter.elm.js
    log::info!("Compiling Reporter.elm.js ...");
    let reporter_template = include_template!("Reporter.elm");
    let reporter_elm_path = tests_root.join("src").join("Reporter.elm");
    std::fs::write(&reporter_elm_path, reporter_template)
        .context("Error writing Reporter.elm to test folder")?;
    let compiled_reporter = tests_root.join("js").join("Reporter.elm.js");
    // let compile_time = std::time::Instant::now();
    if !crate::make::compile(
        elm_home,
        &tests_root,            // current_dir
        &make_options.compiler, // compiler
        &compiled_reporter,     // output
        &[&reporter_elm_path],
    )?
    .success()
    {
        return Ok(1);
    }

    // Generate a package.json specifying that all JS files follow CommonJS.
    std::fs::write(
        tests_root.join("js").join("package.json"),
        "{type: 'commonjs'}",
    )
    .context("Could not write the commonjs guide package.json")?;

    // Generate the supervisor Node module
    let node_supervisor_template = include_template!("node_supervisor.js");
    let node_supervisor_js_file = tests_root.join("js").join("node_supervisor.js");
    crate::make::create_templated(
        node_supervisor_template, // template
        &node_supervisor_js_file, // output
        &[
            ("{{ workersCount }}", &run_options.workers.to_string()),
            ("{{ initialSeed }}", &run_options.seed.to_string()),
            ("{{ fuzzRuns }}", &run_options.fuzz.to_string()),
            ("{{ reporter }}", &run_options.reporter),
            ("{{ verbosity }}", &make_options.verbosity.to_string()),
            ("{{ globs }}", &serde_json::to_string(&make_options.files).context("Failed to convert the list of tests files passed as CLI arguments to a JSON list")?),
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
    let experimental_arg = if node_version.starts_with(b"v10.") {
        vec!["--experimental-worker"]
    } else {
        vec![]
    };

    // Start the tests supervisor
    log::info!("Starting the supervisor ...");
    let mut supervisor = Command::new("node")
        .args(experimental_arg)
        .arg(node_supervisor_js_file)
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
    log::info!("Running tests ...");
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
    Ok(exit_code.unwrap_or(0))
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
            log::error!("Error attempting to wait for child: {}", e);
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
