//! Install packages to test dependencies.

/// Copy behavior of `elm-test install ...`.
pub fn main(packages: Vec<String>) {
    // Recommend direct usage of elm-json instead
    eprintln!("Not implemented.");
    eprintln!("Please use zwilias/elm-json directly instead.");
    eprintln!("elm-json install --test {}", packages.join(" "));
    std::process::exit(1);
}
