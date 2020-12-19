# Changelog

All notable changes to this project will be documented in this file.

## Unreleased [(diff)][diff-unreleased]

<!-- ## [0.2.0] - (2020-11-19) [(diff)][diff-0.2.0] -->


#### Added

- `CHANGELOG.md` to record important changes.
- Includes a stealthy `--watch` option explaining why it is not needed.

#### Changed

- Do not overwrite the generated `elm.json` for the tests if identical.
  This considerably speeds up the compilation step since the elm
  binary uses timestamps to invalidate cache.

#### Removed


#### Fixed


## [0.1.0] - (2020-12-18)

### Added

- `README.md` as the home page of this repository.
- `LICENSE` code is provided under the BSD 3-Clause license.
- `Cargo.toml` configuration of this Rust project.
- `Cargo.lock` exact dependencies this Rust project.
- `src/` containing all the Rust source code.
- `.gitignore` configured for a Rust project.
- `.gitmodules` git submodules.
- `.github/workflows/` CI to automatically build and test on pull requests.

[0.1.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.1
[diff-unreleased]: https://github.com/mpizenberg/elm-test-rs/compare/v0.1...master
<!-- [diff-0.2.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.1...v0.2 -->
