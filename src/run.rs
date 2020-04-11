pub fn main(help: bool, version: bool, compiler: Option<String>, files: Vec<String>) {
    // The help option is prioritary over the othe options
    if help {
        crate::help::main();
        return;
    // The version option is the second priority
    } else if version {
        println!("{}", std::env!("CARGO_PKG_VERSION"));
        return;
    }
    // Generate the list of modules and their file paths
    todo!();
    // Generate a correct elm.json
    todo!();
    // Compile all test files
    todo!();
    // Find all tests
    todo!();
    // Generate the Runner.elm concatenating all tests
    todo!();
    // Compile the Reporter.elm
    todo!();
    // Generate the supervisor Node module
    todo!();
    // Start the tests supervisor
    todo!();
}
