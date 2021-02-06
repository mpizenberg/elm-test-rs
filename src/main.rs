mod deps;
mod help;
mod init;
mod install;
mod parser;
mod run;
mod utils;

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
    match main_args() {
        Ok(Args::Init) => init::main(),
        Ok(Args::Install { packages }) => install::main(packages),
        Ok(Args::Run(options)) => run::main(options),
        Err(e) => anyhow::bail!("Error: {:?}.", e),
    }
}

/// Function parsing the command line arguments and returning an Args object or an error.
fn main_args() -> Result<Args, Box<dyn std::error::Error>> {
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
) -> Result<Args, Box<dyn std::error::Error>> {
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
        seed: args.opt_value_from_str("--seed")?.unwrap_or(rng_seed),
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

fn free_args_str(free_args: Vec<OsString>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut string_args = Vec::with_capacity(free_args.len());
    for arg in free_args.into_iter() {
        string_args.push(arg.into_string().unwrap());
    }
    Ok(string_args)
}
