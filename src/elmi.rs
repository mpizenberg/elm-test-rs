//! Basically a wrapper module for elmi-to-json for the time being.
//! It reads the compiled .elmi files and extracts exposed tests.

use miniserde::{json, Deserialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Use elmi-to-json as a binary to extract all exposed tests
/// from compiled .elmi files.
pub fn all_tests<P: AsRef<Path>>(
    work_dir: P,
    src_files: &HashSet<PathBuf>,
) -> Result<Vec<TestModule>, String> {
    let output = Command::new("elmi-to-json")
        .arg("--for-elm-test")
        .arg("--elm-version")
        .arg("0.19.1")
        // stdio config
        .current_dir(&work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .expect("command failed to start");
    let str_output = std::str::from_utf8(&output.stdout)
        .map_err(|_| "Output of elmi-to-json is not valid UTF-8".to_string())?;
    let output: ElmiToJsonOutput =
        json::from_str(str_output).map_err(|_| "Deserialization error".to_string())?;
    Ok(output
        .test_modules
        .into_iter()
        // Filter out modules with no test
        .filter(|m| !m.tests.is_empty())
        // Filter out modules not in the list of src_files (must also be canonical)
        .filter(|m| {
            let path =
                work_dir.as_ref().join(&m.path).canonicalize().expect(
                    "There was an issue when retrieving module paths from elmi-to-json output",
                );
            src_files.contains(&path)
        })
        // No need to verify that module names are valid, elm 0.19.1 already verifies that.
        // No need to filter exposed since only exposed values are in elmi files.
        .collect())
}

#[derive(Deserialize, Debug)]
/// Struct mirroring the json result of elmi-to-json --for-elm-test.
struct ElmiToJsonOutput {
    #[serde(rename = "testModules")]
    test_modules: Vec<TestModule>,
}

#[derive(Deserialize, Debug)]
/// Test modules as listed in the json result of elmi-to-json.
pub struct TestModule {
    #[serde(rename = "moduleName")]
    pub module_name: String,
    pub path: String,
    pub tests: Vec<String>,
}
