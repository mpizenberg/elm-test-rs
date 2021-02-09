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
    elm-test-rs [<SUBCOMMAND>]
    elm-test-rs [FLAGS...] [TESTFILES...]
    For example:
        elm-test-rs tests/*.elm

FLAGS:
    --help                       # Print this message and exit
    --version                    # Print version string and exit
    --quiet                      # Reduce amount of stderr logs
    --watch                      # Rerun tests on file changes
    --compiler /path/to/compiler # Use a custom path to an Elm executable (defaults to just elm)
    --project /path/to/elm.json/dir
                                 # Precise the path to the root directory of the project (defaults to current dir)
    --seed integer               # Run with initial fuzzer seed (defaults to random)
    --fuzz integer               # Precise number of iterations of fuzz tests (defaults to 100)
    --workers integer            # Precise number of worker threads (defaults to number of logic cores)
    --filter "substring"         # Keep only the tests whose descriptions contain the given string
    --report console|json|junit|exercism
                                 # Print results to stdout in given the format (defaults to console)
    --connectivity progressive|offline|online-newest|online-oldest
                                 # Connectivity mode (defaults to progessive)
                                 #    offline: elm-test-rs only use installed packages to solve dependencies
                                 #    online-newest: the newest compatible dependencies are picked to run tests
                                 #    online-oldest: the oldest compatible dependencies are picked to run tests
                                 #    progressive: try offline first and if that fails, switch to online-newest

SUBCOMMANDS:
    init               # Initialize tests dependencies and directory
    install [PACKAGES] # Install packages to "test-dependencies" in your elm.json
"#;
