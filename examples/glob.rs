use glob::glob;

/// usage: cargo run --example glob -- "src/**/*" "examples/**/*"
pub fn main() {
    for entry in std::env::args()
        // skip program name
        .skip(1)
        // join globs for each command line argument
        .flat_map(|pattern| glob(&pattern).expect("Failed to read glob pattern"))
    {
        match entry {
            // canonical, absolute form of the path
            // with all intermediate components normalized and symbolic links resolved
            Ok(path) => println!("{:?}", path.canonicalize().expect("woops")),
            Err(e) => println!("{:?}", e),
        }
    }
}
