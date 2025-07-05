mod deps;
mod init;
mod install;
mod logger;
mod make;
mod parser;
mod project;
mod run;
mod utils;

use anyhow::Context;
use clap::{App, AppSettings, Arg, SubCommand};
use pubgrub_dependency_provider_elm::dependency_provider::VersionStrategy;
use std::num::NonZeroU32;

/// Main entry point of elm-test-rs.
fn main() -> anyhow::Result<()> {
    // Arguments available to all subcommands.
    let global_args = vec![
        Arg::with_name("elm-home")
            .long("elm-home")
            .global(true)
            .takes_value(true)
            .value_name("path")
            .env("ELM_HOME")
            .help("Use a custom directory for elm home"),
        Arg::with_name("project")
            .long("project")
            .global(true)
            .default_value(".")
            .value_name("path")
            .help("Path to the root directory of the project"),
        Arg::with_name("offline")
            .long("offline")
            .global(true)
            .help("No network call made by elm-test-rs"),
        Arg::with_name("verbose")
            .short("v")
            .multiple(true)
            .global(true)
            .help("Increase verbosity. Can be used multiple times -vvv"),
    ];
    // Arguments shared with the "make" subcommand.
    let make_args = vec![
        Arg::with_name("watch")
            .long("watch")
            .help("Rerun tests on file changes"),
        Arg::with_name("compiler")
            .long("compiler")
            .default_value("elm")
            .help("Use a custom path to an Elm executable"),
        Arg::with_name("dependencies")
            .long("dependencies")
            .takes_value(true)
            .value_name("strategy")
            .possible_values(&["newest", "oldest"])
            .conflicts_with("offline")
            .help("Choose the newest or oldest compatible dependencies (mostly useful for package authors)"),
        Arg::with_name("report")
            .long("report")
            .default_value("console")
            .possible_value("console")
            .possible_value("consoleDebug")
            .possible_value("json")
            .possible_value("junit")
            .possible_value("exercism")
            .help("Print results to stdout in the given format"),
        Arg::with_name("output")
            .long("output")
            .takes_value(true)
            .value_name("output_path")
            .possible_values(&["/dev/null"])
            .help("This argument is ignored, and only present for compatibility with `elm make --output=/dev/null` for the make subcommand"),
        Arg::with_name("PATH or GLOB")
            .multiple(true)
            .help("Path to a test module, or glob pattern such as tests/*.elm")
    ];
    let run_args = vec![
        Arg::with_name("seed")
            .long("seed")
            .takes_value(true)
            .help("Initial random seed for fuzz tests [default: <random>]"),
        Arg::with_name("fuzz")
            .long("fuzz")
            .default_value("100")
            .value_name("N")
            .help("Number of iterations in fuzz tests"),
        Arg::with_name("workers")
            .long("workers")
            .takes_value(true)
            .value_name("N")
            .help("Number of worker threads [default: <number of logic cores>]"),
        Arg::with_name("filter")
            .long("filter")
            .takes_value(true)
            .value_name("string")
            .help("Keep only tests whose description contains the given string"),
        Arg::with_name("deno")
            .long("deno")
            .help("Rerun tests with Deno instead of Node"),
    ];
    let matches = App::new("elm-test-rs")
        .version(std::env!("CARGO_PKG_VERSION"))
        .args(&global_args)
        .args(&make_args)
        .args(&run_args)
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize tests dependencies and directory")
                .setting(AppSettings::DisableVersion),
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Install packages to \"test-dependencies\" in your elm.json")
                .arg(
                    Arg::with_name("PACKAGE")
                        .multiple(true)
                        .help("Package to install"),
                )
                .setting(AppSettings::DisableVersion),
        )
        .subcommand(
            SubCommand::with_name("make")
                .about("Compile tests modules")
                .args(&make_args)
                .setting(AppSettings::DisableVersion),
        )
        .get_matches();

    // Retrieve the path to the elm home.
    let elm_home = match matches.value_of("elm-home") {
        None => utils::elm_home().context("Elm home not found")?,
        Some(str_path) => {
            // Create the path to make sure it exists.
            std::fs::create_dir_all(str_path)
                .context(format!("{str_path} does not exist and is not writable"))?;
            utils::absolute_path(str_path)?
        }
    };

    // Retrieve the path to the project root directory.
    let elm_project_root = utils::elm_project_root(matches.value_of("project").unwrap())?; // unwrap is fine since project has a default value

    // Set log verbosity.
    let verbosity = matches.occurrences_of("verbose");
    logger::init(verbosity).context("Failed to initialize logger")?;

    match matches.subcommand() {
        ("init", Some(sub_matches)) => init::main(
            elm_home,
            elm_project_root,
            sub_matches.is_present("offline"),
        ),
        ("install", Some(sub_matches)) => {
            let packages: Vec<String> = sub_matches
                .values_of("PACKAGE")
                .into_iter()
                .flatten()
                .map(|s| s.to_string())
                .collect();
            install::main(packages)
        }
        ("make", Some(sub_matches)) => {
            let exit_code =
                make::main(&elm_home, &elm_project_root, get_make_options(sub_matches)?)?;
            std::process::exit(exit_code);
        }
        _ => {
            let make_options = get_make_options(&matches)?;
            let run_options = get_run_options(&matches)?;
            let exit_code = run::main(&elm_home, &elm_project_root, make_options, run_options)?;
            std::process::exit(exit_code);
        }
    }
}

/// Retrieve options related to the make subcommand.
fn get_make_options(arg_matches: &clap::ArgMatches) -> anyhow::Result<make::Options> {
    let connectivity = match (
        arg_matches.is_present("offline"),
        arg_matches.value_of("dependencies"),
    ) {
        (false, None) => deps::ConnectivityStrategy::Progressive,
        (true, None) => deps::ConnectivityStrategy::Offline,
        (true, Some(_)) => anyhow::bail!("--offline is incompatible with --dependencies"),
        (false, Some("newest")) => deps::ConnectivityStrategy::Online(VersionStrategy::Newest),
        (false, Some("oldest")) => deps::ConnectivityStrategy::Online(VersionStrategy::Oldest),
        (false, Some(_)) => anyhow::bail!("Invalid --dependencies value"),
    };

    // Handle relative paths for --compiler
    let mut compiler = arg_matches.value_of("compiler").unwrap().to_string(); // unwrap is fine since compiler has a default value
    let compiler_path = std::path::Path::new(&compiler);
    if compiler_path.components().count() > 1 {
        compiler = utils::absolute_path(compiler_path)?
            .to_str()
            .context("Could not convert to &str")?
            .to_string();
    }

    let report = match arg_matches.value_of("report").unwrap() {
        // unwrap is fine since there is a default value
        "json" => String::from("json"),
        _ => String::from("console"),
    };

    let files: Vec<String> = arg_matches
        .values_of("PATH or GLOB")
        .into_iter()
        .flatten()
        .map(|s| s.to_string())
        .collect();
    Ok(make::Options {
        verbosity: arg_matches.occurrences_of("verbose"),
        watch: arg_matches.is_present("watch"),
        compiler,
        connectivity,
        files,
        report,
    })
}

/// Retrieve options related to the main run command.
fn get_run_options(arg_matches: &clap::ArgMatches) -> anyhow::Result<run::Options> {
    // Use nanoseconds of current time as seed.
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    let seed: u32 = match arg_matches.value_of("seed") {
        None => now.unwrap().as_nanos() as u32,
        Some(str_seed) => str_seed.parse().context("Invalid --seed value")?,
    };
    let str_fuzz = arg_matches.value_of("fuzz").unwrap(); // unwrap is fine since there is a default value
    let fuzz: NonZeroU32 = str_fuzz
        .parse()
        .context("Invalid --fuzz value. It must be a positive integer.")?;

    let workers: u32 = match arg_matches.value_of("workers") {
        None => num_cpus::get() as u32,
        Some(str_workers) => str_workers.parse().context("Invalid --workers value")?,
    };

    let reporter = match arg_matches.value_of("report").unwrap() {
        // unwrap is fine since there is a default value
        "console" => String::from(console_color_mode()),
        r => String::from(r),
    };

    let runtime = if arg_matches.is_present("deno") {
        run::Runtime::Deno
    } else {
        run::Runtime::Node
    };
    Ok(run::Options {
        seed,
        fuzz,
        workers,
        filter: arg_matches.value_of("filter").map(|s| s.to_string()),
        reporter,
        runtime,
    })
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
