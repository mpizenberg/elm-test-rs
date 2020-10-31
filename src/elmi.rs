//! Basically a wrapper module for elmi-to-json for the time being.
//! It reads the compiled .elmi files and extracts exposed tests.

use std::fs;

use std::path::Path;

/// Find all possible tests (all values) in test_files.
pub fn all_tests(
    test_files: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<Vec<TestModule>, String> {
    test_files
        .into_iter()
        .map(|test_file| {
            let source = fs::read_to_string(&test_file).unwrap();

            let tree = {
                let mut parser = tree_sitter::Parser::new();
                let language = super::parser::tree_sitter_elm();
                parser.set_language(language).unwrap();
                parser.parse(&source, None).unwrap()
            };

            crate::parser::get_all_exposed_values(&tree, &source)
                .map(|tests| TestModule {
                    path: test_file.as_ref().to_str().unwrap().to_string(),
                    tests: tests.iter().map(|s| s.to_string()).collect(),
                })
                .map_err(|s| s.to_string())
        })
        .collect()
}

pub struct TestModule {
    pub path: String,
    pub tests: Vec<String>,
}
