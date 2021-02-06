//! Install packages to test dependencies.

/// Copy behavior of `elm-test install ...`.
pub fn main(packages: Vec<String>) -> anyhow::Result<()> {
    // Recommend direct usage of elm-json instead
    anyhow::bail!(
        r#"
Not implemented. Please use zwilias/elm-json directly instead.
elm-json install --test {}"#,
        packages.join(" ")
    )
}
