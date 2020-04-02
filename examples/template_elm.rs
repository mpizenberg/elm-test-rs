use std::collections::HashMap;
use varj;

/// cargo run --example template_elm
pub fn main() {
    // String template of elm main file.
    // Could be loaded from a file on disk instead.
    let elm_main_template: String = r#"
{{ user_imports }}
import Test.Reporter.Reporter exposing (Report(..))
import Console.Text exposing (UseColor(..))
import Test.Runner.Node
import Test

main : Test.Runner.Node.TestProgram
main =
    [ {{ tests }} ]
        |> Test.concat
        |> Test.Runner.Node.run {{ opt_code }}
"#
    .to_string();

    // Hash map containing values for all keys {{ key }}
    // present in the previous template.
    let replacements: HashMap<String, String> = vec![
        ("user_imports".to_string(), user_test_modules().join("\n")),
        ("tests".to_string(), user_exposed_tests().join(", ")),
        ("opt_code".to_string(), opt_code()),
    ]
    .into_iter()
    .collect();

    println!(
        "{}",
        // Use the varj to replace all {{ key }} in template `elm_main_template`
        // by the corresponding value provided in the hash map `replacements`.
        // Return an error if there are missing keys.
        varj::parse(&elm_main_template, &replacements).unwrap()
    );
}

/// Simulate a list of test modules provided by some other code
fn user_test_modules() -> Vec<String> {
    vec![
        "Some.Test".to_string(),
        "Some.Other.Test".to_string(),
        "Some.Third.Test".to_string(),
    ]
    .into_iter()
    .map(|module| format!("import {}", module))
    .collect()
}

/// Simulate a list of exposed tests provided by some other code
fn user_exposed_tests() -> Vec<String> {
    vec![
        "Some.Test.test1".to_string(),
        "Some.Other.Test.test2".to_string(),
        "Some.Third.Test.test3".to_string(),
    ]
}

/// Simulate a config record provided by some other code
fn opt_code() -> String {
    r#"{ some = "record", with = 44 }"#.to_string()
}
