# Tests for elm-test-rs

## End-to-end tests on example projects

The `example-projects/` directory contains many small projects aimed
at being tested on CI to catch regressions.
They are all tested within the `end-to-end.rs` test file
so they are checked with all other tests when running `cargo test`.

They are organized in the following sub directories:
1. `passing`: tests should pass.
2. `erroring`: elm-test-rs should error with an exit code of 1.
3. `failing`: tests should fail with an exit code of 2.

### Passing

- `app`: minimal app with one passing test.
- `pkg`: minimal package with one passing test.
- `es-module`: contains a `package.json` file specifying that JS files should be considered as ES module.

### Erroring (exit code 1)

- `no-test-module`: no test module.
- `missing-src-dir`: missing `src/` directory.

### Failing (exit code 2)

- `no-test`: no exposed test.
- `todo`: a todo is present in the tests.
