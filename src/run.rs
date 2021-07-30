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
    pub runtime: Runtime,
}

#[derive(Debug)]
/// The runtime to be used.
pub enum Runtime {
    /// Node is the default runtime.
    Node,
    /// Deno is an alternative runtime.
    Deno,
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
    let es_module = match run_options.runtime {
        Runtime::Node => false,
        Runtime::Deno => true,
    };
    fs::write(
        &compiled_runner,
        &kernel_patch_tests(&compiled_runner_src, es_module).context(format!(
            "Failed to patch the file {}",
            compiled_runner.display()
        ))?,
    )
    .context(format!(
        "Failed to write the patched file {}",
        compiled_runner.display()
    ))?;

    // Generate the node_runner.js node module embedding the Elm runner

    let (runner_name, runner_template) = match run_options.runtime {
        Runtime::Node => ("node_runner.js", include_template!("node_runner.js")),
        Runtime::Deno => ("deno_runner.mjs", include_template!("deno_runner.mjs")),
    };
    let polyfills = include_template!("node_polyfills.js");
    let runner_path = tests_root.join("js").join(runner_name);
    let filter = match &run_options.filter {
        None => "null".to_string(),
        Some(s) => format!("\"{}\"", s),
    };
    crate::make::create_templated(
        runner_template, // template
        &runner_path,    // output
        &[
            ("{{ initialSeed }}", &run_options.seed.to_string()),
            ("{{ fuzzRuns }}", &run_options.fuzz.to_string()),
            ("{{ filter }}", &filter),
            ("{{ polyfills }}", polyfills),
        ],
    )
    .context(format!("Failed to write {}", runner_path.display()))?;

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

    // For a Deno runtime, convert the compiled Reporter.elm.js into an ES module.
    if let Runtime::Deno = run_options.runtime {
        let compiled_reporter_code = fs::read_to_string(&compiled_reporter)?;
        fs::write(
            &compiled_reporter,
            into_es_module(&replace_console_log(&compiled_reporter_code)),
        )?;
    }

    // Generate a package.json specifying that all JS files follow CommonJS.
    std::fs::write(
        tests_root.join("js").join("package.json"),
        "{type: 'commonjs'}",
    )
    .context("Could not write the commonjs guide package.json")?;

    // Generate the supervisor Node module
    let (supervisor_name, supervisor_template) = match run_options.runtime {
        Runtime::Node => (
            "node_supervisor.js",
            include_template!("node_supervisor.js"),
        ),
        Runtime::Deno => (
            "deno_supervisor.mjs",
            include_template!("deno_supervisor.mjs"),
        ),
    };
    let supervisor_js_file = tests_root.join("js").join(supervisor_name);
    crate::make::create_templated(
        supervisor_template, // template
        &supervisor_js_file, // output
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
        supervisor_js_file.display()
    ))?;

    // For a Deno runtime, make deno_linereader.mjs available.
    if let Runtime::Deno = run_options.runtime {
        let linereader_template = include_template!("deno_linereader.mjs");
        let linereader_path = tests_root.join("js").join("deno_linereader.mjs");
        std::fs::write(linereader_path, linereader_template)?;
    }

    // Start the tests supervisor
    log::info!("Starting the supervisor ...");
    let mut supervisor = match run_options.runtime {
        Runtime::Node => {
            let node_version = Command::new("node")
                .arg("--version")
                .output()
                .context("\"node --version\" failed to start")?
                .stdout;

            // Node supports worker_threads as experimental feature since 10.5,
            // but it is unknown whether all versions since 10.5 actually work with elm-test-rs.
            let experimental_arg = if node_version.starts_with(b"v10.") {
                Some("--experimental-worker")
            } else {
                None
            };

            Command::new("node")
                .args(experimental_arg)
                .arg(supervisor_js_file)
                .current_dir(tests_root)
                .stdin(Stdio::piped())
                .spawn()
                .context("Node supervisor failed to start")?
        }
        Runtime::Deno => Command::new("deno")
            .args(["run", "--allow-read", "--allow-hrtime"])
            .arg(supervisor_js_file)
            .current_dir(tests_root)
            .stdin(Stdio::piped())
            .spawn()
            .context("Deno supervisor failed to start")?,
    };

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
    let runner_path_string = runner_path
        .to_str()
        .context(format!(
            "Could not convert path into a String: {}",
            runner_path.display()
        ))?
        .to_string();
    writeln(runner_path_string.as_bytes())
        .context("Failed to write runner path to supervisor stdin")?;

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
///
/// Also replace the unique call to console.log in Debug.log
/// by a call to the "yet-to-be-defined" console.elmlog
///
/// Transformation to an esmodule is also possible.
fn kernel_patch_tests(elm_js: &str, esmodule: bool) -> anyhow::Result<String> {
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
        test_variant_definition.replace_all(elm_js, "$0 __elmTestSymbol: __elmTestSymbol,");
    let elm_js = check_definition.replace(&elm_js, "$1 = value => value && value.__elmTestSymbol === __elmTestSymbol ? $$elm$$core$$Maybe$$Just(value) : $$elm$$core$$Maybe$$Nothing;");

    let elm_js = ["const __elmTestSymbol = Symbol('elmTestSymbol');", &elm_js].join("\n");

    // If an ES module is asked, the following transformation is applied.
    if esmodule {
        Ok(into_es_module(&replace_console_log(&elm_js)))
    } else {
        Ok(replace_console_log(&elm_js))
    }
}

/// Replace console.log with console.elmlog and remove console.warn.
fn replace_console_log(elm_js: &str) -> String {
    // WARNING: this may fail if a user has this as a string somewhere
    // and it is located before its definition by elm in the file.
    let elm_js = elm_js.replacen(
        "console.log(tag + ': ' + _Debug_toString(value));",
        "console.elmlog(tag + ': ' + _Debug_toString(value));",
        1,
    );
    // Remove the console.warn() at the begining due to not compiling with --optimize
    elm_js.replacen("console.warn", "", 1)
}

/// Convert an JS file resulting from an Elm compilation into an ES module.
fn into_es_module(elm_js: &str) -> String {
    // replace '}(this));' by '}(scope));' at the end.
    let last_this_offset = elm_js.rfind("this").unwrap();
    [
        "const scope = {};",
        &elm_js[..last_this_offset],
        "scope));",
        "export const { Elm } = scope;",
    ]
    .join("\n")
}
