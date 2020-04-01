use seahorse::{App, Flag, FlagType};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let app = App::new()
        .name("elm-test-rs")
        .author(std::env!("CARGO_PKG_AUTHORS"))
        .description(std::env!("CARGO_PKG_DESCRIPTION"))
        .usage("elm-test-rs [command] [arg]")
        .version(std::env!("CARGO_PKG_VERSION"))
        .action(default)
        .flag(Flag::new("version", "elm-test-rs --version(-v)", FlagType::Bool).alias("v"));
    app.run(args);
}

fn default(context: &seahorse::Context) {
    if context.bool_flag("version") {
        print_version();
    } else {
        println!("{:?}", context.args);
    }
}

fn print_version() {
    println!("{}", std::env!("CARGO_PKG_VERSION"));
}
