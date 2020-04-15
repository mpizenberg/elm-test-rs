pub fn main() {
    println!("{}", USAGE);
}

const USAGE: &'static str = r#"
elm-test-rs
An alternative Elm test runner to node-test-runner

USAGE:
    elm-test-rs [<SUBCOMMAND>] [FLAGS] [TESTFILES]
    For example:
        elm-test-rs tests/*.elm

FLAGS:
    --help                       # Print this message and exit
    --version                    # Print version string and exit
    --compiler /path/to/compiler # Use given version of elm
    --seed integer               # Run with initial fuzzer seed
    --fuzz integer               # Precise number of iterations of fuzz tests
    --workers integer            # Precise number of worker threads
    --report console|json|junit  # Print results to stdout in given format (default to console)
    (--watch)                    # (Not supported yet) Run tests on file changes

SUBCOMMANDS:
    init               # Initialize tests dependencies and directory
    install [PACKAGES] # Install packages to "test-dependencies" in your elm.json
"#;
