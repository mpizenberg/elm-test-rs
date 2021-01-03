//! Initialize elm tests.

use pubgrub_dependency_provider_elm::project_config::ProjectConfig;

use crate::include_template;

/// Add elm-explorations/test to test dependencies
/// and initialize a template tests/Tests.elm file.
pub fn main() {
    // Install elm-explorations/test in the tests dependencies
    let elm_json_str = std::fs::read_to_string("elm.json").expect("Unable to read elm.json");
    let project_config: ProjectConfig = serde_json::from_str(&elm_json_str).unwrap();
    let updated_config = crate::deps::init(project_config)
        .expect("Something went wrong when installing elm-explorations/test");
    std::fs::write(
        "elm.json",
        serde_json::to_string_pretty(&updated_config).unwrap(),
    )
    .expect("Unable to write to updated elm.json");

    // Create the tests/Tests.elm template
    let init_tests_template = include_template!("Tests.elm");
    std::fs::create_dir_all("tests").expect("Impossible to create directory tests/");
    let new_file_path = std::path::Path::new("tests").join("Tests.elm");
    if !new_file_path.exists() {
        std::fs::write(new_file_path, init_tests_template)
            .expect("Unable to create Tests.elm template");
        eprintln!("The file test/Tests.elm was created");
    }
}
