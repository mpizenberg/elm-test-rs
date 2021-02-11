mod deps;
mod help;
mod init;
mod install;
mod parser;
mod run;
mod utils;

use anyhow::Context;
use clap::{App, AppSettings, Arg, SubCommand};
use std::ffi::OsString;

#[derive(Debug)]
/// Type representing command line arguments.
enum Args {
    Init,
    Install { packages: Vec<String> },
    Run(run::Options),
}

/// Main entry point of elm-test-rs.
fn main() -> anyhow::Result<()> {
    let matches = App::new("elm-test-rs")
        .version(std::env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("quiet")
                .long("quiet")
                .help("Reduce amount of stderr logs"),
        )
        .arg(
            Arg::with_name("watch")
                .long("watch")
                .help("Rerun tests on file changes"),
        )
        .arg(
            Arg::with_name("elm-home")
                .long("elm-home")
                .global(true)
                .takes_value(true)
                .value_name("path")
                .env("ELM_HOME")
                .help("Use a custom directory for elm home"),
        )
        .arg(
            Arg::with_name("compiler")
                .long("compiler")
                .default_value("elm")
                .help("Use a custom path to an Elm executable"),
        )
        .arg(
            Arg::with_name("project")
                .long("project")
                .global(true)
                .default_value(".")
                .value_name("path")
                .help("Path to the root directory of the project"),
        )
        .arg(
            Arg::with_name("seed")
                .long("seed")
                .takes_value(true)
                .help("Initial random seed for fuzz tests [default: <random>]"),
        )
        .arg(
            Arg::with_name("fuzz")
                .long("fuzz")
                .default_value("100")
                .value_name("N")
                .help("Number of iterations in fuzz tests"),
        )
        .arg(
            Arg::with_name("workers")
                .long("workers")
                .takes_value(true)
                .value_name("N")
                .help("Number of worker threads [default: <number of logic cores>]"),
        )
        .arg(
            Arg::with_name("filter")
                .long("filter")
                .takes_value(true)
                .value_name("string")
                .help("Keep only tests whose description contains the given string"),
        )
        .arg(
            Arg::with_name("report")
                .long("report")
                .default_value("console")
                .possible_value("console")
                .possible_value("consoleDebug")
                .possible_value("consoleColor")
                .possible_value("consoleNoColor")
                .possible_value("json")
                .possible_value("junit")
                .possible_value("exercism")
                .help("Print results to stdout in the given format"),
        )
        .arg(
            Arg::with_name("offline")
                .long("offline")
                .help("No network call made by elm-test-rs"),
        )
        .arg(
            Arg::with_name("dependencies")
                .long("dependencies")
                .takes_value(true)
                .value_name("strategy")
                .possible_values(&["newest", "oldest"])
                .conflicts_with("offline")
                .help("Choose the newest or oldest compatible dependencies (mostly useful for package authors)"),
        )
        .arg(
            Arg::with_name("PATH or GLOB")
                .multiple(true)
                .help("Path to a test module, or glob pattern such as tests/*.elm")
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize tests dependencies and directory")
                .setting(AppSettings::DisableVersion)
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Install packages to \"test-dependencies\" in your elm.json")
                .setting(AppSettings::DisableVersion)
        )
        .get_matches();
    Ok(())
}

/// Main entry point of elm-test-rs.
fn main_() -> anyhow::Result<()> {
    match main_args().context("There was an error while parsing CLI arguments.")? {
        Args::Init => init::main(
            utils::elm_home().context("Elm home not found")?,
            utils::elm_project_root("")?,
        ),
        Args::Install { packages } => install::main(packages),
        Args::Run(options) => run::main(options),
    }
}

/// Function parsing the command line arguments and returning an Args object or an error.
fn main_args() -> anyhow::Result<Args> {
    let mut args = pico_args::Arguments::from_env();
    match args.subcommand()?.as_deref() {
        Some("init") => Ok(Args::Init),
        Some("install") => Ok(Args::Install {
            packages: free_args_str(args.finish())?,
        }),
        // The first arg may be mistaken for an unknown subcommand
        Some(first_arg) => no_subcommand_args(Some(first_arg.to_string()), args),
        None => no_subcommand_args(None, args),
    }
}

/// Parse all command options and file arguments.
/// first_arg is here in case it was mistaken for an unknown subcommand
/// and will be prepended to the rest of free arguments.
/// This happens for example with the command: `elm-test-rs /path/to/some/Module.elm`.
fn no_subcommand_args(
    first_arg: Option<String>,
    mut args: pico_args::Arguments,
) -> anyhow::Result<Args> {
    // Use nanoseconds of current time as seed.
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    let rng_seed = now.unwrap().as_nanos() as u32;

    Ok(Args::Run(run::Options {
        help: args.contains("--help"),
        version: args.contains("--version"),
        quiet: args.contains("--quiet"),
        watch: args.contains("--watch"),
        compiler: args
            .opt_value_from_str("--compiler")?
            .unwrap_or_else(|| "elm".to_string()),
        project: args
            .opt_value_from_str("--project")?
            .unwrap_or_else(|| ".".to_string()),
        seed: args
            .opt_value_from_str("--seed")
            .context("Invalid argument for --seed")?
            .unwrap_or(rng_seed),
        fuzz: args.opt_value_from_str("--fuzz")?.unwrap_or(100),
        workers: args
            .opt_value_from_str("--workers")?
            .unwrap_or(num_cpus::get() as u32),
        filter: args.opt_value_from_str("--filter")?,
        report: args
            .opt_value_from_str("--report")?
            .unwrap_or_else(|| "console".to_string()),
        connectivity: args
            .opt_value_from_str("--connectivity")?
            .unwrap_or(deps::ConnectivityStrategy::Progressive),
        files: {
            let mut files = free_args_str(args.finish())?;
            if let Some(file) = first_arg {
                files.insert(0, file);
            }
            files
        },
    }))
}

fn free_args_str(free_args: Vec<OsString>) -> anyhow::Result<Vec<String>> {
    let mut string_args = Vec::with_capacity(free_args.len());
    for arg in free_args.into_iter() {
        string_args.push(arg.into_string().unwrap());
    }
    Ok(string_args)
}
