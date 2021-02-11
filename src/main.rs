mod deps;
mod init;
mod install;
mod parser;
mod run;
mod utils;

use anyhow::Context;
use clap::{App, AppSettings, Arg, SubCommand};
use pubgrub_dependency_provider_elm::dependency_provider::VersionStrategy;
use std::path::PathBuf;

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
                .arg(Arg::with_name("PACKAGE").multiple(true).help("Package to install"))
                .setting(AppSettings::DisableVersion)
        )
        .get_matches();

    // Retrieve the path to the elm home.
    let elm_home = match matches.value_of("elm-home") {
        None => utils::elm_home().context("Elm home not found")?,
        Some(str_path) => PathBuf::from(str_path),
    };

    // Retrieve the path to the project root directory.
    let elm_project_root = utils::elm_project_root(matches.value_of("project").unwrap())?; // unwrap is fine since project has a default value

    match matches.subcommand() {
        ("init", Some(_)) => init::main(elm_home, elm_project_root),
        ("install", Some(sub_matches)) => {
            let packages: Vec<String> = sub_matches
                .values_of("PACKAGE")
                .into_iter()
                .flatten()
                .map(|s| s.to_string())
                .collect();
            install::main(packages)
        }
        _ => {
            // Use nanoseconds of current time as seed.
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
            let seed: u32 = match matches.value_of("seed") {
                None => now.unwrap().as_nanos() as u32,
                Some(str_seed) => str_seed.parse().context("Invalid --seed value")?,
            };
            let str_fuzz = matches.value_of("fuzz").unwrap(); // unwrap is fine since there is a default value
            let fuzz: u32 = str_fuzz.parse().context("Invalid --fuzz value")?;
            let workers: u32 = match matches.value_of("workers") {
                None => num_cpus::get() as u32,
                Some(str_workers) => str_workers.parse().context("Invalid --workers value")?,
            };
            let connectivity = match (
                matches.is_present("offline"),
                matches.value_of("dependencies"),
            ) {
                (false, None) => deps::ConnectivityStrategy::Progressive,
                (true, None) => deps::ConnectivityStrategy::Offline,
                (true, Some(_)) => anyhow::bail!("--offline is incompatible with --dependencies"),
                (false, Some("newest")) => {
                    deps::ConnectivityStrategy::Online(VersionStrategy::Newest)
                }
                (false, Some("oldest")) => {
                    deps::ConnectivityStrategy::Online(VersionStrategy::Oldest)
                }
                (false, Some(_)) => anyhow::bail!("Invalid --dependencies value"),
            };
            let files: Vec<String> = matches
                .values_of("PATH or GLOB")
                .into_iter()
                .flatten()
                .map(|s| s.to_string())
                .collect();
            let options = run::Options {
                quiet: matches.is_present("quiet"),
                watch: matches.is_present("watch"),
                compiler: matches.value_of("compiler").unwrap().to_string(), // unwrap is fine since compiler has a default value
                seed,
                fuzz,
                workers,
                filter: matches.value_of("filter").map(|s| s.to_string()),
                report: matches.value_of("report").unwrap().to_string(), // unwrap is fined since there is a default value
                connectivity,
                files,
            };
            run::main(&elm_home, &elm_project_root, options)
        }
    }
}
