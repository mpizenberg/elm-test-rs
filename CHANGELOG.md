# Changelog

All notable changes to this project will be documented in this file.


## Unreleased [(diff)][diff-unreleased]

#### Added

- A `--quiet` CLI option. Currently it reduces the amount of stderr logging.

#### Changed

- Split direct and indirect dependencies after dependency resolution
  to avoid naming issues when importing modules.

#### Removed

#### Fixed


## [0.5.1] - (2021-01-05) [(diff)][diff-0.5.1]

#### Added

- `utils::include_template!` macro for easier logic and maintainance.

#### Changed

- Use `utils::include_template!` macro for easier logic and maintainance,
  instead of duplicated calls to `include_str!` with a `unix` or `windows` guard.

#### Fixed

- Update to version 3.1.1 of elm code that fixes some things in exercism report.


## [0.5.0] - (2020-12-31) [(diff)][diff-0.5.0]

#### Added

- Support Node 10 with `--experimental-worker` option.
- Exercism support with the "exercism" value for `--report` option.

#### Fixed

- Fixed parser issue for long unicode chars.


## [0.4.1] - (2020-12-25) [(diff)][diff-0.4.1]

#### Changed

- Only display startup debug logs if there are any.


## [0.4.0] - (2020-12-23) [(diff)][diff-0.4.0]

#### Added

- Now handles tests runs of the kind "Only" | "Skipping" | "Invalid".
- Support for colors with the ConsoleColor reporter.
- Nice formatting of failure diffs like in node-test-runner.
- Ability to pick connectivity with version stratey: "offline" | "online-newest" | "online-oldest".
- Add a --filter option to only run some tests based on their description.

#### Changed

- Renamed the "Console" reporter into "ConsoleDebug" reporter.

#### Fixed

- Exit code error.
- Await termination of runners before exiting.

## [0.3.0] - (2020-12-21) [(diff)][diff-0.3.0]

#### Added

- Capture `Debug.log` calls happening when running a test.

#### Changed

- Rename all occurences of ...Nb... into ...Count, for example:
  `askNbTests` becomes `askTestsCount`,
  `sendNbTests` becomes `sendTestsCount`,
  `nb_workers` becomes `workersCount`, etc.
- Simplfy the Elm/JS interop code between the runners and supervisor.


## [0.2.0] - (2020-12-20) [(diff)][diff-0.2.0]

#### Added

- Include a simple `--watch` option for convenience.

#### Fixed

- Swap the `dirs` crate (unmaintained) by `dirs-next`.


## [0.1.1] - (2020-12-19) [(diff)][diff-0.1.1]

#### Added

- `CHANGELOG.md` to record important changes.
- Includes a stealthy `--watch` option explaining why it is not needed.
- Print elm-test-rs version to stderr at the beginning of output.

#### Changed

- Do not overwrite the generated `elm.json` for the tests if identical.
  This considerably speeds up the compilation step since the elm
  binary uses timestamps to invalidate cache.

#### Removed

- Dependency to lazy_static


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

[0.5.1]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.5.1
[0.5.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.5
[0.4.1]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.4.1
[0.4.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.4
[0.3.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.3
[0.2.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.2
[0.1.1]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.1
[diff-unreleased]: https://github.com/mpizenberg/elm-test-rs/compare/v0.5.1...master
[diff-0.5.1]: https://github.com/mpizenberg/elm-test-rs/compare/v0.5...v0.5.1
[diff-0.5.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.4.1...v0.5
[diff-0.4.1]: https://github.com/mpizenberg/elm-test-rs/compare/v0.4...v0.4.1
[diff-0.4.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.3...v0.4
[diff-0.3.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.2...v0.3
[diff-0.2.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.1.1...v0.2
[diff-0.1.1]: https://github.com/mpizenberg/elm-test-rs/compare/v0.1...v0.1.1
