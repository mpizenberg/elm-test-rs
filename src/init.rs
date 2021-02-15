//! Initialize elm tests.

use crate::include_template;
use anyhow::Context;
use pubgrub_dependency_provider_elm::project_config::ProjectConfig;
use std::path::Path;

/// Add elm-explorations/test to test dependencies
/// and initialize a template tests/Tests.elm file.
pub fn main<P: AsRef<Path>>(elm_home: P, project_root: P, offline: bool) -> anyhow::Result<()> {
    // Install elm-explorations/test in the tests dependencies
    let project_root = project_root.as_ref();
    let elm_json_path = project_root.join("elm.json");
    let elm_json_str =
        std::fs::read_to_string(&elm_json_path).context("Unable to read elm.json")?;
    let project_config: ProjectConfig =
        serde_json::from_str(&elm_json_str).context("Invalid elm.json")?;
    let updated_config = crate::deps::init(elm_home, project_config, offline).context(
        "Something went wrong when installing elm-explorations/test to the tests dependencies",
    )?;
    crate::utils::json_write(&elm_json_path, &updated_config)
        .context("Unable to write the updated elm.json")?;

    // Create the tests/Tests.elm template
    let init_tests_template = include_template!("Tests.elm");
    let tests_dir = project_root.join("tests");
    std::fs::create_dir_all(&tests_dir).context("Impossible to create directory tests/")?;
    let new_file_path = tests_dir.join("Tests.elm");
    if !new_file_path.exists() {
        std::fs::write(new_file_path, init_tests_template)
            .context("Unable to create Tests.elm template")?;
        log::warn!("The file tests/Tests.elm was created");
    }
    Ok(())
}
