use assert_cmd::Command;
use std::path::Path;

// -------------------------------------------------------------------
// Checking that the error code returned are correct on all projects
// -------------------------------------------------------------------

#[test]
fn check_all_passing() {
    for entry in
        std::fs::read_dir(Path::new("tests").join("example-projects").join("passing")).unwrap()
    {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_example(&path, 0);
        }
    }
}

#[test]
fn check_all_erroring() {
    for entry in
        std::fs::read_dir(Path::new("tests").join("example-projects").join("erroring")).unwrap()
    {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_example(&path, 1);
        }
    }
}

#[test]
fn check_all_failing() {
    for entry in
        std::fs::read_dir(Path::new("tests").join("example-projects").join("failing")).unwrap()
    {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_example(&path, 2);
        }
    }
}

fn check_example(project_dir: &Path, exit_code: i32) {
    let mut cmd = Command::cargo_bin("elm-test-rs").unwrap();
    let assert = cmd.current_dir(project_dir).arg("-vvv").assert();
    assert.code(exit_code);
}

// -------------------------------------------------------------------
// Testing that the --project CLI argument works as expected
// -------------------------------------------------------------------

// #[test]
fn check_arg_project() {
    let passing = Path::new("tests").join("example-projects").join("passing");
    let app = passing.join("app");
    let pkg = passing.join("pkg");
    // Checking --project for the app
    let mut cmd = Command::cargo_bin("elm-test-rs").unwrap();
    cmd.arg("--project").arg(app).arg("-vvv").assert().success();
    // Checking --project for the package
    let mut cmd = Command::cargo_bin("elm-test-rs").unwrap();
    cmd.arg("--project").arg(pkg).arg("-vvv").assert().success();
}
