//! Print manually crafted help message.
//! Automatic help is not handled by the lightweight pico-args.

/// Print help message.
pub fn main() {
    println!("{}", USAGE);
}

const USAGE: &str = r#"
elm-test-rs
An alternative Elm test runner to node-test-runner

USAGE:
    elm-test-rs [<SUBCOMMAND>] [FLAGS] [TESTFILES]
    For example:
        elm-test-rs tests/*.elm

FLAGS:
    --help                       # Print this message and exit
    --version                    # Print version string and exit
    --watch                      # Rerun tests on file changes
    --compiler /path/to/compiler # Precis the compiler to use (defaults to just elm)
    --seed integer               # Run with initial fuzzer seed (defaults to random)
    --fuzz integer               # Precise number of iterations of fuzz tests (defaults to 100)
    --workers integer            # Precise number of worker threads (defaults to number of logic cores)
    --report console|json|junit  # Print results to stdout in given format (defaults to console)

SUBCOMMANDS:
    init               # Initialize tests dependencies and directory
    install [PACKAGES] # Install packages to "test-dependencies" in your elm.json
"#;
