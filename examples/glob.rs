use glob::glob;
use std::collections::HashSet;
use std::path::PathBuf;

/// usage: cargo run --example glob -- "src/**/*" "examples/**/*"
pub fn main() {
    let files: HashSet<PathBuf> = std::env::args()
        // skip program name
        .skip(1)
        // join globs for each command line argument
        .flat_map(|pattern| glob(&pattern).expect("Failed to read glob pattern"))
        // filter out errors
        .filter_map(|x| x.ok())
        // canonicalize in absolute form
        .map(|x| x.canonicalize().expect("Error in canonicalize"))
        // collect into a set of unique values
        .collect();
    files.iter().for_each(|f| println!("{:?}", f));
}
