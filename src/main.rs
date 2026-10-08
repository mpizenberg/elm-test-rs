mod cli;
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
use cli::{Command, Dependencies, ElmTestRs, MakeArgs, Report, RunArgs};
use pubgrub_dependency_provider_elm::dependency_provider::VersionStrategy;
use std::io::IsTerminal;

enum Action {
    Init(init::Options),
    Make(make::Options),
    Run(make::Options, run::Options),
}

/// Main entry point of elm-test-rs.
fn main() -> anyhow::Result<()> {
    let ElmTestRs { global, command } = parse_args();
    let verbosity = global.verbose.into();
    let compiler = get_compiler(&global.compiler)?;

    // Check the options before doing anything with the project.
    let action = match command {
        // The install subcommand only prints a recommendation.
        Command::Install(install) => return install::main(install.packages),
        Command::Completions(completions) => {
            let shell =
                usage::complete::Shell::from_name(&completions.shell).context("Unknown shell")?;
            print!("{}", ElmTestRs::completion_script(shell));
            return Ok(());
        }
        Command::Init => Action::Init(init::Options { compiler }),
        Command::Make(make) => {
            Action::Make(get_make_options(make, global.offline, verbosity, compiler)?)
        }
        Command::Run(run) => {
            let run_options = get_run_options(&run);
            let make_options = get_make_options(run.make, global.offline, verbosity, compiler)?;
            Action::Run(make_options, run_options)
        }
    };

    // Retrieve the path to the elm home.
    let elm_home = match global.elm_home {
        None => utils::elm_home().context("Elm home not found")?,
        Some(path) => {
            // Create the path to make sure it exists.
            std::fs::create_dir_all(&path).context(format!(
                "{} does not exist and is not writable",
                path.display()
            ))?;
            utils::absolute_path(path)?
        }
    };

    // Retrieve the path to the project root directory.
    let elm_project_root = utils::elm_project_root(&global.project)?;

    // Set log verbosity.
    logger::init(verbosity).context("Failed to initialize logger")?;

    match action {
        Action::Init(init_options) => {
            init::main(elm_home, elm_project_root, global.offline, init_options)
        }
        Action::Make(make_options) => {
            let exit_code = make::main(&elm_home, &elm_project_root, make_options)?;
            std::process::exit(exit_code);
        }
        Action::Run(make_options, run_options) => {
            let exit_code = run::main(&elm_home, &elm_project_root, make_options, run_options)?;
            std::process::exit(exit_code);
        }
    }
}

/// Parse the command line arguments, or print the help, version, or usage error and exit.
fn parse_args() -> ElmTestRs {
    let argv: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    match ElmTestRs::embedded_outcome(&argv) {
        usage::embedded::Outcome::Parsed(cli) => cli,
        usage::embedded::Outcome::Exit(exit) => {
            if exit.stderr {
                eprint!("{}", exit.text);
            } else {
                print!("{}", exit.text);
            }
            // Exit code 2 is reserved for failing tests, so usage errors exit with 1.
            std::process::exit(exit.code.min(1));
        }
    }
}

/// Retrieve options related to the make subcommand.
fn get_make_options(
    args: MakeArgs,
    offline: bool,
    verbosity: u64,
    compiler: String,
) -> anyhow::Result<make::Options> {
    let connectivity = match (offline, args.dependencies) {
        (false, None) => deps::ConnectivityStrategy::Progressive,
        (true, None) => deps::ConnectivityStrategy::Offline,
        (true, Some(_)) => anyhow::bail!("--offline is incompatible with --dependencies"),
        (false, Some(Dependencies::Newest)) => {
            deps::ConnectivityStrategy::Online(VersionStrategy::Newest)
        }
        (false, Some(Dependencies::Oldest)) => {
            deps::ConnectivityStrategy::Online(VersionStrategy::Oldest)
        }
    };

    let report = match args.report {
        Report::Json => String::from("json"),
        _ => String::from("console"),
    };

    Ok(make::Options {
        verbosity,
        watch: args.watch,
        compiler,
        connectivity,
        files: args.files,
        report,
    })
}

/// Retrieve options related to the main run command.
fn get_run_options(args: &RunArgs) -> run::Options {
    let reporter = match args.make.report {
        Report::Console => console_color_mode(),
        Report::ConsoleDebug => "consoleDebug",
        Report::Json => "json",
        Report::Junit => "junit",
        Report::Exercism => "exercism",
    };

    let runtime = if args.deno {
        run::Runtime::Deno
    } else {
        run::Runtime::Node
    };
    run::Options {
        seed: args.seed,
        fuzz: args.fuzz,
        workers: args.workers.get(),
        filter: args.filter.clone(),
        reporter: String::from(reporter),
        runtime,
    }
}

/// Retrieve the path to the Elm compiler, resolving relative paths to absolute
/// ones (a bare command such as "elm" is left as-is to be looked up in PATH).
fn get_compiler(compiler: &str) -> anyhow::Result<String> {
    let compiler_path = std::path::Path::new(compiler);
    if compiler_path.components().count() > 1 {
        Ok(utils::absolute_path(compiler_path)?
            .to_str()
            .context("Could not convert to &str")?
            .to_string())
    } else {
        Ok(compiler.to_string())
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
            std::io::stdout().is_terminal(),
            std::env::var("CLICOLOR").as_deref(),
        ) {
            (false, _) => "consoleNoColor",
            (true, Ok("0")) => "consoleNoColor",
            (true, _) => "consoleColor",
        }
    }
}
