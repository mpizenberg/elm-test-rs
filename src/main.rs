use seahorse::{App, Command, Flag, FlagType};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let app = App::new()
        .name("elm-test-rs")
        .author(std::env!("CARGO_PKG_AUTHORS"))
        .description(std::env!("CARGO_PKG_DESCRIPTION"))
        .usage("elm-test-rs [command] [arg]")
        .version(std::env!("CARGO_PKG_VERSION"))
        .action(default_action)
        .flag(Flag::new("version", "elm-test-rs --version(-v)", FlagType::Bool).alias("v"))
        .flag(Flag::new(
            "compiler",
            "elm-test-rs --compiler /path/to/elm",
            FlagType::String,
        ))
        .command(init_command())
        .command(install_command());
    app.run(args);
}

fn default_action(context: &seahorse::Context) {
    if context.bool_flag("version") {
        print_version();
    } else if let Some(elm_path) = context.string_flag("compiler") {
        println!("compiler: {}", elm_path);
        run(context);
    } else {
        run(context);
    }
}

fn run(context: &seahorse::Context) {
    println!("TODO: main program");
    println!("args: {:?}", context.args);
}

fn print_version() {
    println!("{}", std::env!("CARGO_PKG_VERSION"));
}

fn init_command() -> Command {
    Command::new()
        .name("init")
        .usage("elm-test-rs init")
        .action(init_action)
}

fn init_action(context: &seahorse::Context) {
    println!("TODO: init command");
    println!("args: {:?}", context.args);
}

fn install_command() -> Command {
    Command::new()
        .name("install")
        .usage("elm-test-rs install package [package]")
        .action(install_action)
}

fn install_action(context: &seahorse::Context) {
    println!("TODO: install command");
    println!("args: {:?}", context.args);
}
