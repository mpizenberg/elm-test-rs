use std::path::PathBuf;

fn main() {
    let dir: PathBuf = ["tree-sitter-elm", "src"].iter().collect();

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .flag_if_supported("-Wno-unused-but-set-variable")
        .compile("tree-sitter-parser");

    cc::Build::new()
        .cpp(true)
        .include(&dir)
        .file(dir.join("scanner.cc"))
        .compile("tree-sitter-scanner");
}
