//! Initialize elm tests.

use crate::include_template;
use crate::project::Project;
use anyhow::Context;
use std::path::Path;

/// Add elm-explorations/test to test dependencies
/// and initialize a template tests/Tests.elm file.
pub fn main<P: AsRef<Path>>(elm_home: P, project_root: P, offline: bool) -> anyhow::Result<()> {
    // Install elm-explorations/test in the tests dependencies
    let project = Project::from_dir(project_root)?;
    let updated_config = crate::deps::init(elm_home, project.config, offline).context(
        "Something went wrong when installing elm-explorations/test to the tests dependencies",
    )?;
    crate::utils::json_write(project.root_directory.join("elm.json"), &updated_config)
        .context("Unable to write the updated elm.json")?;

    // Create the tests/Tests.elm template
    let init_tests_template = include_template!("Tests.elm");
    let tests_dir = project.root_directory.join("tests");
    std::fs::create_dir_all(&tests_dir).context("Impossible to create directory tests/")?;
    let new_file_path = tests_dir.join("Tests.elm");
    if !new_file_path.exists() {
        std::fs::write(new_file_path, init_tests_template)
            .context("Unable to create Tests.elm template")?;
        log::error!("The file tests/Tests.elm was created");
    }
    Ok(())
}
