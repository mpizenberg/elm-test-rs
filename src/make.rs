//! Module dealing with compiling the test code.

use anyhow::Context;
use glob::glob;
use pubgrub_dependency_provider_elm::project_config::ProjectConfig;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

use crate::include_template;
use crate::project::Project;

#[derive(Debug)]
/// Options passed as arguments.
pub struct Options {
    pub verbosity: u64,
    pub watch: bool,
    pub compiler: String,
    pub connectivity: crate::deps::ConnectivityStrategy,
    pub files: Vec<String>,
    pub report: String,
}

/// Main function, generating and compiling a Runner.elm file.
/// It has multiple steps that can be summarized as:
///
///  1. Generate the list of test modules and their file paths.
///  2. Generate a correct `elm.json` for the to-be-generated `Runner.elm`.
///  3. Find all tests.
///  4. Generate `Runner.elm` with a master test concatenating all found exposed tests.
///  5. Compile it.
pub fn main(elm_home: &Path, elm_project_root: &Path, options: Options) -> anyhow::Result<i32> {
    // Prints to stderr the current version
    let title = format!(
        "elm-test-rs {} for elm 0.19.1",
        std::env!("CARGO_PKG_VERSION")
    );
    log::warn!("\n{}\n{}\n", &title, "-".repeat(title.len()));

    let mut project = Project::from_dir(elm_project_root.to_path_buf())?;
    if options.watch {
        project.watch(|proj| main_helper(elm_home, proj, &options).map(|_| ()))?;
        Ok(0)
    } else {
        match main_helper(elm_home, &project, &options)? {
            Output::MakeFailure => Ok(1),
            Output::MakeSuccess { .. } => Ok(0),
        }
    }
}

/// Output of running "elm make" on all the tests files.
pub enum Output {
    MakeFailure,
    MakeSuccess {
        tests_root: PathBuf,
        modules_abs_paths: HashSet<PathBuf>,
        compiled_runner: PathBuf,
    },
}

/// Do main stuff and outputs the paths to the tests directories
/// (useful for watch mode).
pub fn main_helper(
    elm_home: &Path,
    project: &Project,
    options: &Options,
) -> anyhow::Result<Output> {
    let start_time = std::time::Instant::now();

    let modules_abs_paths = if options.files.is_empty() {
        // Default with elm modules in the tests/ directory
        elm_files_within(project.root_directory.join("tests"))
            .map(crate::utils::absolute_path)
            .collect::<Result<_, _>>()?
    } else {
        // Get file paths of all modules in canonical form (absolute path)
        get_elm_modules_abs_paths(&options.files)?
    };

    // Report an error if no file was found.
    if modules_abs_paths.is_empty() {
        if options.files.is_empty() {
            anyhow::bail!("No file was found in your tests/ directory. You can create one with: elm-test-rs init");
        } else {
            anyhow::bail!(
                "No file was found matching your pattern: {}",
                options.files.join(" ")
            );
        }
    }

    let tests_root = project
        .root_directory
        .join("elm-stuff")
        .join("tests-0.19.1");
    // Make src dirs relative to the generated tests root
    let source_directories_for_runner = project
        .src_and_test_dirs
        .iter()
        .map(|path| {
            pathdiff::diff_paths(&path, &tests_root).context(format!(
                "Could not get path {} relative to path {}",
                path.display(),
                tests_root.display()
            ))
        })
        .chain(
            // Add src/ to the source directories for Runner.elm
            std::iter::once(Ok("src".into())),
        )
        .collect::<Result<Vec<PathBuf>, _>>()?;

    // Generate an elm.json for the to-be-generated Runner.elm.
    log::info!("Generating the elm.json for the Runner.elm");
    let tests_config = crate::deps::solve(
        elm_home,
        &options.connectivity,
        &project.config,
        source_directories_for_runner.as_slice(),
    )
    .context("Failed to solve dependencies for tests to run")?;
    log::info!(
        "The dependencies picked to run the tests are:\n{}",
        serde_json::to_string_pretty(&tests_config.dependencies)
            .context("Failed to convert to JSON the picked dependencies")?,
    );
    let tests_config = ProjectConfig::Application(tests_config);
    let tests_config_path = tests_root.join("elm.json");
    std::fs::create_dir_all(tests_root.join("src")).context(format!(
        "Could not create tests dir {}",
        tests_root.join("src").display()
    ))?;
    // If it has changed, update the elm.json for the tests.
    let tests_config_str = serde_json::to_string(&tests_config)
        .context("Failed to convert to JSON the string generated for the tests elm.json")?;
    match std::fs::read_to_string(&tests_config_path) {
        Ok(old_conf) if tests_config_str == old_conf => (),
        _ => std::fs::write(tests_config_path, tests_config_str)
            .context("Unable to write to generated elm.json")?,
    };

    // Find module names
    let mut module_names = Vec::new();
    for p in modules_abs_paths.iter() {
        module_names.push(get_module_name(&project.src_and_test_dirs, p)?);
    }

    // Runner.elm imports of tests modules
    let imports: Vec<String> = module_names
        .iter()
        .map(|m| format!("import {}", m))
        .collect();

    // Find all potential tests
    log::info!("Finding all potential tests ...");
    let mut potential_tests = Vec::new();
    for (module_name, path) in module_names.iter().zip(&modules_abs_paths) {
        let source =
            fs::read_to_string(&path).context(format!("Failed to read {}", path.display()))?;
        for potential_test in crate::parser::potential_tests(&source) {
            potential_tests.push(format!("check {}.{}", module_name, potential_test));
        }
    }

    // Generate templated src/Runner.elm
    let runner_template = include_template!("Runner.elm");
    let runner_elm_file = tests_root.join("src").join("Runner.elm");
    create_templated(
        runner_template,  // template
        &runner_elm_file, // output
        &[
            ("{{ imports }}", &imports.join("\n")),
            ("{{ potential_tests }}", &potential_tests.join("\n    , ")),
        ],
    )
    .context(format!("Failed to write {}", runner_elm_file.display()))?;

    // Compile the src/Runner.elm file into Runner.elm.js
    let _preparation_time = start_time.elapsed().as_secs_f32();
    log::info!("Spent {}s generating Runner.elm", _preparation_time);
    log::info!("Compiling the generated templated src/Runner.elm ...");
    let compiled_runner = tests_root.join("js").join("Runner.elm.js");
    let command = compile(
        elm_home,
        &tests_root,       // current_dir
        &options.compiler, // compiler
        &compiled_runner,  // output
        &options.report,   // report
        &[Path::new("src").join("Runner.elm")],
    )?;
    if command.status.success() {
        log::warn!("✓ Compilation of tests modules succeeded");
        Ok(Output::MakeSuccess {
            tests_root,
            modules_abs_paths,
            compiled_runner,
        })
    } else {
        // Always put the json output of `elm make` to stdout to be consistent
        // with the fact that the tests runner output also goes to stdout.
        match options.report.as_str() {
            "json" => std::io::stdout().write_all(&command.stderr),
            _ => std::io::stderr().write_all(&command.stderr),
        }?;
        Ok(Output::MakeFailure)
    }
}

/// List recursively all elm files within a given directory.
fn elm_files_within<P: AsRef<Path>>(directory: P) -> impl Iterator<Item = PathBuf> {
    let walker = WalkDir::new(directory).follow_links(true);
    let entries = walker.into_iter().filter_map(|e| e.ok());
    entries.map(|e| e.into_path()).filter(|p| is_elm_file(&p))
}

fn is_elm_file<P: AsRef<Path>>(p: P) -> bool {
    p.as_ref().extension() == Some(OsStr::new("elm"))
}

/// Collect absolute paths of all elm files matching the patterns given as arguments.
fn get_elm_modules_abs_paths(args: &[String]) -> anyhow::Result<HashSet<PathBuf>> {
    let mut glob_err = Ok(());
    let abs_paths: HashSet<PathBuf> = args
        .iter()
        .map(|arg| resolve_glob_arg(arg))
        .scan(&mut glob_err, |err, res| {
            res.map_err(|e| **err = Err(e)).ok()
        })
        .flatten()
        .map(|path| absolute_elm_path(&path))
        .collect::<Result<_, _>>()?;
    glob_err.map_err(anyhow::Error::from)?;
    Ok(abs_paths)
}

/// If the argument is a path to an existing file,
/// return an iterator with just this file.
/// Otherwise, interpret it as a glob pattern and resolve it into a file iterator.
fn resolve_glob_arg(arg: &str) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    let path = PathBuf::from(arg);
    if path.exists() {
        Ok(either::Left(std::iter::once(path)))
    } else {
        resolve_glob_pattern(arg).map(either::Right)
    }
}

fn resolve_glob_pattern(pattern: &str) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    Ok(glob(pattern)
        .context(format!("Failed to read glob pattern {}", pattern))?
        .filter_map(|gr| gr.ok()))
}

/// Transform path into an absolute path and check that it is an elm file.
fn absolute_elm_path(path: &Path) -> anyhow::Result<PathBuf> {
    if is_elm_file(path) {
        crate::utils::absolute_path(path)
    } else {
        anyhow::bail!("{} isn't an elm file", path.display())
    }
}

/// Compile an Elm module into a JS file (without --optimized)
pub fn compile<P1, P2, I, S>(
    elm_home: &Path,
    current_dir: P1,
    compiler: &str,
    output: P2,
    report: &str,
    src: I,
) -> anyhow::Result<std::process::Output>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let context_if_fails = format!(
        r#"
Failed to run {}. Are you sure it's in your PATH?
If you installed elm locally with npm, maybe try running with npx such as:

    npx --no-install elm-test-rs"#,
        compiler
    );
    let output = output.as_ref().to_str().context(format!(
        "Could not convert path into a String: {}",
        output.as_ref().display()
    ))?;
    log::debug!(
        "Running \"{}\" with current_dir set to \"{}\"",
        compiler,
        current_dir.as_ref().display()
    );
    let executable = which::CanonicalPath::new(compiler).context(context_if_fails.clone())?;
    let executable = executable.as_path();
    log::debug!("We found an executable: {}", executable.display());
    if executable.extension() == Some(OsStr::new("cmd")) {
        shell_command(
            elm_home,
            compiler,
            src,
            output,
            current_dir.as_ref(),
            report,
        )
        .context(context_if_fails)
    } else {
        // Transform the --report argument into an argument for the elm compiler.
        let report_arg = match report {
            "json" => Some("--report=json"),
            _ => None,
        };
        // Capture compiler output if --report=json.
        let stderr = match report {
            "json" => Stdio::piped(),
            _ => Stdio::inherit(),
        };
        Command::new(executable)
            .env("ELM_HOME", elm_home)
            .arg("make")
            .arg(format!("--output={}", output))
            .args(report_arg)
            .args(src)
            .current_dir(current_dir)
            // stdio config, comment to see elm make output for debug
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(stderr)
            .output()
            .context(context_if_fails)
    }
}

/// Uses cmd to execute a command on Windows
#[cfg(windows)]
fn shell_command<I, S>(
    elm_home: &Path,
    compiler: &str,
    src: I,
    output: &str,
    current_dir: &Path,
    report: &str,
) -> Result<std::process::Output, std::io::Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    log::debug!("Trying with a cmd shell");
    // Transform the --report argument into an argument for the elm compiler.
    let report_arg = match report {
        "json" => Some("--report=json"),
        _ => None,
    };
    // Capture compiler output if --report=json.
    let stderr = match report {
        "json" => Stdio::piped(),
        _ => Stdio::inherit(),
    };
    Command::new("cmd")
        .env("ELM_HOME", elm_home)
        .arg("/D")
        .arg("/Q")
        .arg("/C")
        .arg(compiler)
        .arg("make")
        .arg(format!("--output={}", output))
        .args(report_arg)
        .args(src)
        .current_dir(current_dir)
        // stdio config, comment to see elm make output for debug
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr)
        .output()
}

/// Fails on unix
#[cfg(unix)]
#[allow(unused_variables)]
fn shell_command<I, S>(
    elm_home: &Path,
    compiler: &str,
    src: I,
    output: &str,
    current_dir: &Path,
    report: &str,
) -> Result<std::process::Output, std::io::Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(compiler).output()
}

/// Replace the template keys and write result to output file.
pub fn create_templated<P: AsRef<Path>>(
    template: &str,
    output: P,
    replacements: &[(&str, &str)],
) -> Result<(), std::io::Error> {
    let mut output_str = template.to_string();
    replacements
        .iter()
        .for_each(|(from, to)| output_str = output_str.replacen(from, to, 1));
    std::fs::write(output, output_str)
}

/// By finding the module name from the file path we can import it even if
/// the file is full of errors. Elm will then report what’s wrong.
fn get_module_name(
    source_dirs: impl IntoIterator<Item = impl AsRef<Path>>,
    file: impl AsRef<Path>,
) -> anyhow::Result<String> {
    log::debug!("get_module_name of: {}", file.as_ref().display());
    let file = file.as_ref();
    let matching_source_dir = {
        let mut matching = source_dirs.into_iter().filter(|dir| file.starts_with(dir));
        match (matching.next(), matching.next()) {
            (None, None) => {
                anyhow::bail!(
                    "This file \"{}\" matches no source directory! Imports won’t work then.",
                    file.display()
                )
            }
            (Some(dir), None) => dir,
            _ => anyhow::bail!("2+ matching source dirs for file \"{}\".", file.display()),
        }
    };

    let trimmed: PathBuf = file
        .strip_prefix(&matching_source_dir)
        .context(format!(
            "Error while trimming prefix \"{}\" of \"{}\".",
            matching_source_dir.as_ref().display(),
            file.display()
        ))?
        .with_extension("");
    let mut module_name_parts = Vec::new();
    for s in trimmed.iter() {
        module_name_parts.push(
            s.to_str()
                .context(format!("Failed to convert path to string: {:?}", s))?,
        );
    }
    module_name_parts
        .iter()
        .filter(|s| !is_valid_module_name(s))
        .for_each(|s| log::debug!("This part is not valid for a module name: {}", s));
    if !module_name_parts.iter().all(|s| is_valid_module_name(s)) {
        anyhow::bail!("I could not guess the module name of {} from its trimmed path {}. It may contains invalid parts.", file.display(), trimmed.display());
    }
    if module_name_parts.is_empty() {
        anyhow::bail!(
            "There is something wrong with the file path of {}",
            file.display()
        );
    }
    Ok(module_name_parts.join("."))
}

fn is_valid_module_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().next().unwrap().is_uppercase() // unwrap() is fine here
        && name.chars().all(|c| c == '_' || c.is_alphanumeric())
}
