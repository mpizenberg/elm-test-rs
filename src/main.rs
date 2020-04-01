use pico_args;

#[derive(Debug)]
enum Args {
    Init,
    Install {
        packages: Vec<String>,
    },
    Global {
        help: bool,
        version: bool,
        compiler: Option<String>,
        files: Vec<String>,
    },
}

fn main() {
    match main_args() {
        Ok(args) => println!("{:?}", args),
        Err(e) => eprintln!("Error: {:?}.", e),
    }
}

fn main_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();
    match args.subcommand()?.as_deref() {
        Some("init") => Ok(Args::Init),
        Some("install") => Ok(Args::Install {
            packages: args.free()?,
        }),
        // The first arg may be mistaken for an unknown subcommand
        Some(first_arg) => no_subcommand_args(Some(first_arg.to_string()), args),
        None => no_subcommand_args(None, args),
    }
}

/// first_arg is here in case it was mistaken for an unknown subcommand
/// and will be prepended to the rest of free arguments.
/// This happens for example with `elm-test-rs /path/to/some/Module.elm`.
fn no_subcommand_args(
    first_arg: Option<String>,
    args: pico_args::Arguments,
) -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = args;
    Ok(Args::Global {
        help: args.contains("--help"),
        version: args.contains("--version"),
        compiler: args.opt_value_from_str("--compiler")?,
        files: {
            let mut files = args.free()?;
            if let Some(file) = first_arg {
                files.insert(0, file);
            }
            files
        },
    })
}
