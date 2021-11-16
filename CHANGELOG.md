# Changelog

All notable changes to this project will be documented in this file.


## Unreleased [(diff)][diff-unreleased]

#### Added

- Added support for the `--report` flag in the `elm-test-rs make` subcommand.
  The stderr stream of the json report is redirected to stdout for consistency in elm-test-rs.
- Accept (and ignore) the `--output=/dev/null` flag to ease integration with tooling and editors.
- Improved compatibility with Lamdera.

#### Changed

- Upgrade `assert_cmd` dependency to replace an unmaintained indirect dependency.

#### Removed

- Removed dependency to stderrlog due to RUSTSEC-2020-0071.

#### Fixed

- (BREAKING) Fix an error in the json report: `testsCompleted` -> `testCompleted`.
- Fix an error in the `package.json` specifying that the generated code is commonjs.
- Fix CI cross compilation to generate the Apple M1 executable of `elm-test-rs`.

## [1.2.2] - (2021-09-18) [(diff)][diff-1.2.2]

#### Fixed

- Added a delimiter between runs in stderr output when detecting a file change in --watch mode.
- Fixed the missing newline for Debug.log capture.
  This bug was introduced in version 1.2.0 due to changes for the Deno compatibility.


## [1.2.1] - (2021-09-08) [(diff)][diff-1.2.1]

#### Fixed

- Actually changed the version indicator of the program to 1.2.1.
  I had forgotten it before so version 1.2 indicates 1.1 with `--version`.


## [1.2.0] - (2021-09-08) [(diff)][diff-1.2.0]

#### Added

- Added arm precompiled binaries to the GitHub action.
- Added a Deno runtime available via the `--deno` CLI argument.
  This also results in a slight change in the way calls to `Debug.log` are captured, but that should be transparent to the user.

#### Fixed

- Better support for situations with symlinks.
  There was an issue when elm files were symlinks to directories outside of the source directories of the elm project.


## [1.1.0] - (2021-06-23) [(diff)][diff-1.1.0]

#### Added

- Added a `task_id` field in the exercism report.


## [1.0.0] - (2021-02-27) [(diff)][diff-1.0.0]

#### Added

- New `--project` CLI option to be able to pass as argument the path to the root
  of the elm project, containing the `elm.json`.
- The globs CLI arguments and resulting found tests modules paths
  are now passed as arguments `globs` and `paths` to the reporter flags.
- New `--offline` flag instead of old `--connectivity` to run offline.
- New `--dependencies [newest|oldest]` argument.
- New `make` subcommand that stops after compilation of the tests modules.
- New `--elm-home` argument also able to pick up the env variable `ELM_HOME`.
- New multiple verbosity flag `-v` to replace the previous `--quiet` flag.
  Level 0 (default) has mostly quiet stderr, level 3 `-vvv` add debug info,
  and levels in between add more and more info.

#### Changed

- Http timeouts are increased from 1s to 10s.
- Updated the Elm submodule for the test runner.
- Changed the CLI crate from pico_args to clap.

#### Removed

- The `--connectivity` argument is no more.
  Replaced by a combination of `--offline` and `--dependencies <strategy>`.
- The `--quiet` flag, replaced by verbosity `-vvv`.

#### Fixed

- All unwise usage of `.unwrap()` and `.expect()` has been replaced
  by correct error handling thanks to the `anyhow` crate.
- Fix some Junit and JSON reports issues.
- Fix `--compiler` error when using relative paths.
- Fix the indentation of generated elm.json. Now uses 4 spaces.
- Fix the order of packages in the elm.json dependencies.
- Add a message to stderr and fail when no test was found.
- Validate that `--fuzz` is >= 1.
- Fix race condition in `--watch` mode preventing a re-run if files change while running tests.
- Fix Windows paths issues due to UNC paths.
- Fix elm command for Windows when installed via npm.


## [0.6.1] - (2021-01-23) [(diff)][diff-0.6.1]

#### Fixed

- Add `elm/json` to the direct dependencies.


## [0.6.0] - (2021-01-22) [(diff)][diff-0.6.0]

#### Added

- A `--quiet` CLI option. Currently it reduces the amount of stderr logging.

#### Changed

- Update pico-args dependency from 0.3.4 to 0.4.
- Update regex dependency from 1.4.1 to 1.4.3.
- Update serde_json dependency from 1.0.59 to 1.0.61.
- Update ureq dependency from 1.5.2 to 2.0.1.

#### Fixed

- Split direct and indirect dependencies after dependency resolution
  to avoid naming issues when importing modules.


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

[1.2.2]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v1.2.2
[1.2.1]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v1.2.1
[1.2.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v1.2
[1.1.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v1.1
[1.0.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v1.0
[0.6.1]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.6.1
[0.6.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.6
[0.5.1]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.5.1
[0.5.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.5
[0.4.1]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.4.1
[0.4.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.4
[0.3.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.3
[0.2.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.2
[0.1.1]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/mpizenberg/elm-test-rs/releases/tag/v0.1
[diff-unreleased]: https://github.com/mpizenberg/elm-test-rs/compare/v1.2.2...master
[diff-1.2.2]: https://github.com/mpizenberg/elm-test-rs/compare/v1.2.1...v1.2.2
[diff-1.2.1]: https://github.com/mpizenberg/elm-test-rs/compare/v1.2...v1.2.1
[diff-1.2.0]: https://github.com/mpizenberg/elm-test-rs/compare/v1.1...v1.2
[diff-1.1.0]: https://github.com/mpizenberg/elm-test-rs/compare/v1.0...v1.1
[diff-1.0.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.6.1...v1.0
[diff-0.6.1]: https://github.com/mpizenberg/elm-test-rs/compare/v0.6...v0.6.1
[diff-0.6.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.5.1...v0.6
[diff-0.5.1]: https://github.com/mpizenberg/elm-test-rs/compare/v0.5...v0.5.1
[diff-0.5.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.4.1...v0.5
[diff-0.4.1]: https://github.com/mpizenberg/elm-test-rs/compare/v0.4...v0.4.1
[diff-0.4.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.3...v0.4
[diff-0.3.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.2...v0.3
[diff-0.2.0]: https://github.com/mpizenberg/elm-test-rs/compare/v0.1.1...v0.2
[diff-0.1.1]: https://github.com/mpizenberg/elm-test-rs/compare/v0.1...v0.1.1
