//! Declaration of the command line interface.

use std::num::NonZeroU32;
use std::path::PathBuf;
use usage::{Args, Cli, Subcommands, ValueEnum};

/// Simple and fast Rust alternative to node-test-runner to run elm tests
#[derive(Cli)]
#[usage(
    bin = "elm-test-rs",
    version,
    completion,
    unknown_flags = "error",
    default_subcommand = "run",
    default_subcommand_flags,
    default_subcommand_on_empty,
    default_subcommand_help
)]
pub struct ElmTestRs {
    #[usage(flatten)]
    pub global: GlobalArgs,
    #[usage(subcommand)]
    pub command: Command,
}

#[derive(Subcommands)]
pub enum Command {
    /// Compile and run the tests
    Run(RunArgs),
    /// Initialize tests dependencies and directory
    Init,
    /// Install packages to "test-dependencies" in your elm.json
    Install(InstallArgs),
    /// Compile tests modules
    Make(MakeArgs),
    /// Print the shell completion script
    Completions(CompletionsArgs),
}

/// Arguments available to all subcommands.
#[derive(Args)]
pub struct GlobalArgs {
    /// Use a custom directory for elm home
    #[usage(
        long,
        env = "ELM_HOME",
        value_name = "path",
        value_hint = usage::ValueHint::DirPath,
        global
    )]
    pub elm_home: Option<PathBuf>,
    /// Path to the root directory of the project
    #[usage(
        long,
        default = ".",
        value_name = "path",
        value_hint = usage::ValueHint::DirPath,
        global
    )]
    pub project: String,
    /// Use a custom path to an Elm executable
    #[usage(
        long,
        default = "elm",
        value_name = "path",
        value_hint = usage::ValueHint::ExecutablePath,
        global
    )]
    pub compiler: String,
    /// No network call made by elm-test-rs
    #[usage(long, global)]
    pub offline: bool,
    /// Increase verbosity. Can be used multiple times -vvv
    #[usage(short = 'v', count, global)]
    pub verbose: u8,
}

#[derive(Args)]
pub struct InstallArgs {
    /// Package to install
    #[usage(name = "PACKAGE")]
    pub packages: Vec<String>,
}

#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate the completion script for
    #[usage(choices("bash", "elvish", "fish", "nu", "powershell", "zsh"))]
    pub shell: String,
}

/// Arguments shared by the main command and the make subcommand.
#[derive(Args)]
pub struct MakeArgs {
    /// Rerun tests on file changes
    #[usage(long)]
    pub watch: bool,
    /// Choose the newest or oldest compatible dependencies (mostly useful for package authors)
    #[usage(long, value_enum, value_name = "strategy")]
    pub dependencies: Option<Dependencies>,
    /// Print results to stdout in the given format
    #[usage(long, value_enum, default = "console", value_name = "report")]
    pub report: Report,
    /// This argument is ignored, and only present for compatibility with `elm make --output=/dev/null` for the make subcommand
    #[usage(long, choices("/dev/null"), value_name = "output_path")]
    pub output: Option<String>,
    /// Path to a test module, or glob pattern such as tests/*.elm
    #[usage(
        name = "PATH or GLOB",
        value_hint = usage::ValueHint::FilePath,
        extensions("elm")
    )]
    pub files: Vec<String>,
}

/// Arguments of the main command, when running the tests.
#[derive(Args)]
pub struct RunArgs {
    #[usage(flatten)]
    pub make: MakeArgs,
    /// Initial random seed for fuzz tests
    #[usage(
        long,
        default_fn = random_seed,
        default_note = "<random>",
        value_name = "seed"
    )]
    pub seed: u32,
    /// Number of iterations in fuzz tests
    #[usage(long, default = "100", value_name = "N")]
    pub fuzz: NonZeroU32,
    /// Number of worker threads
    #[usage(
        long,
        default_fn = logic_cores,
        default_note = "<number of logic cores>",
        value_name = "N"
    )]
    pub workers: NonZeroU32,
    /// Keep only tests whose description contains the given string
    #[usage(long, value_name = "string")]
    pub filter: Option<String>,
    /// Rerun tests with Deno instead of Node
    #[usage(long)]
    pub deno: bool,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Dependencies {
    Newest,
    Oldest,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Report {
    Console,
    #[usage(name = "consoleDebug")]
    ConsoleDebug,
    Json,
    Junit,
    Exercism,
}

/// Use nanoseconds of current time as seed.
fn random_seed() -> u32 {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    now.unwrap().as_nanos() as u32
}

fn logic_cores() -> NonZeroU32 {
    std::thread::available_parallelism()
        .ok()
        .and_then(|n| NonZeroU32::new(n.get() as u32))
        .unwrap_or(NonZeroU32::MIN)
}
