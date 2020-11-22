# elm-test-rs

Attempt at a simpler alternative to node-test-runner for elm tests.


## Usage

Just replace `elm-test` by `elm-test-rs`.
Currently, you need to have elm 0.19.1, [elm-json][elm-json] installed.

[elm-json]: https://github.com/zwilias/elm-json


## Install

There is no installation yet, you need to build the tool
and add **a link** to it in a directory in your PATH env variable.
Beware that copying the executable (instead of symlink) will not work
since currently, it needs to find some template files at runtime.
To build the `elm-test-rs` binary, install Rust and run the command:

```sh
cargo build
```

The executable will be located at `target/debug/elm-test-rs`.


## Design goals

The objective is to get an easy to maintain and extend test runner.
For these reasons, the core design goals are for the code to be

- as simple and lightweight as reasonably possible,
- modular,
- well documented.


## Features

The aim is to provide at least feature parity with elm-test
plus few other nice additions.
However, this doesn't intend to support Elm prior to 0.19.1.

Missing features for parity with elm-test:

 - [ ] `--watch` mode ([issue #7][watch-mode])
 - [ ] colors ([issue #5][colors])
 - [ ] pretty-printing of diffs ([issue #6][pretty-printing])
 - [ ] timing of runs

Additional features:

 - [x] `--workers` option to choose the number of runner workers
 - [ ] capturing `Debug.log` calls ([example implementation][capture-log])
 - [ ] progess bar ([example implementation][progress-bar])

[watch-mode]: https://github.com/mpizenberg/elm-test-rs/issues/7
[colors]: https://github.com/mpizenberg/elm-test-rs/issues/5
[pretty-printing]: https://github.com/mpizenberg/elm-test-rs/issues/6
[capture-log]: https://github.com/mpizenberg/elm-test-rs/pull/4
[progress-bar]: https://github.com/mpizenberg/elm-test-rs/pull/3


## Code architecture

The code of this project is split in three parts.

 1. The CLI, a rust application that generates all the needed JS and Elm files to run tests.
 2. The supervisor, a small Node JS script
    (roughly 100 lines, no dependency other than Node itself)
    tasked to spawn runners (Elm), start a reporter (Elm)
    and transfer tests results from the runners to the reporter.
 3. An Elm package (pure, no debug logging) [mpizenberg/elm-test-runner][elm-test-runner]
    exposing a main program for a runner and one for a reporter.

The supervisor and the runners communicate through child and parent worker messages.
The reporter is just loaded as a Node module by the supervisor.
Communication between the Elm and JS parts are done through ports, as usual.
More details about the supervisor, runner and reporter parts are available
in [mpizenberg/elm-test-runner][elm-test-runner].

[elm-test-runner]: elm

Rust was chosen for the first part since it is a very well fitted language
for systemish CLI programs and enables consise, fast and robust programs.
But any other language could replace this since it is completely independent
from the supervisor, runner and reporter code.
Communication between the CLI and supervisor is assumed to go through STDIN and STDOUT
so no need to lose your hair on weird platform-dependent issues
with inter-process-communication (IPC) going through named pipes.
The CLI program, if asked to run the tests, performs the following actions.

 1. Generate the list of test modules and their file paths.
 2. Generate a correct `elm.json` for the to-be-generated `Runner.elm`.
 3. Compile all test files such that we know they are correct.
 4. Find all tests.
 5. Generate `Runner.elm` with a master test concatenating all found exposed tests.
 6. Compile it into a JS file wrapped into a Node worker module.
 7. Compile `Reporter.elm` into a Node module.
 8. Generate and start the Node supervisor program.

![architecture diagram][diagram]

[diagram]: https://mpizenberg.github.io/resources/elm-test-rs/elm-test-rs.png


## Contributing

Contributions are very welcome.
This project uses [rust format][rustfmt] and [clippy][clippy] (with its default options) to enforce good code style.
To install these tools run

```bash
rustup update
rustup component add clippy rustfmt
```

And then before committing run

```bash
cargo fmt
cargo clippy
```

PS: clippy is a rapidly evolving tool so if there are lint errors on CI
don't forget to `rustup update`.

[rustfmt]: https://github.com/rust-lang/rustfmt
[clippy]: https://github.com/rust-lang/rust-clippy


## Shortcuts and improvements

As this is still a proof of concept, I cut a few corners to get things working.
For example, the generation of the `elm.json` for the tests uses directly
[zwilias/elm-json][elm-json] as a binary.

Eventually, it would be useful to extract the dependency solving algorithm from elm-json
into a crate of its own and to make it available offline if a suitable solution
is possible with already installed packages.
The solver code in elm-json has been extracted from [elba][elba],
a package manager for the Idris language.
It is iself a rust implementation of [PubGrub][pubgrub],
the version solver for the Dart language.
A very nice introduction to PubGrub is given in a [blog post][pubgrub] of 2018
by its author, Natalie Weizenbaum.

[elba]: https://github.com/elba/elba
[pubgrub]: https://medium.com/@nex3/pubgrub-2fb6470504f


## LOC

Total lines of code for elm-test-rs (including elm-test-runner Elm package)
as of commit aa360eed:

| Language    |  Files   |  Lines   |  Code   |  Comments  |
| ----------- | --------:| --------:| -------:| ----------:|
| Elm         |     12   |   1268   |   713   |       220  |
| JavaScript  |      3   |    199   |   151   |        23  |
| Rust        |      8   |    712   |   542   |       108  |


## Embedding template files in the executable?

Currently, the Rust CLI program uses `std::env::current_exe()`
to find the location of the `elm-test-rs` executable in order do find
the needed files at runtime such as templates and Elm files to compile.
Publishing the Elm package in [mpizenberg/elm-test-runner][elm-test-runner],
would remove the problem of finding the Elm files but we still need the templates.
An option would be to use [rust-embed][rust-embed] to load those files
directly into the executable.
This way, elm-test-rs would be perfectly portable.

[rust-embed]: https://github.com/pyros2097/rust-embed


## Cross compilation for OSX and Windows (TODO)

- Look at configuration of [BurntSushi/ripgrep][ripgrep]
- Look at configuration of [zwilias/elm-json][elm-json]
- Discussion on Rust forum: [Cross compile macOS and MS Windows][forum-cross]
- Medium post by Dotan Nahum:
  [Building Rust for Multiple Platforms Using Github Actions][medium-github-action]

[ripgrep]: https://github.com/BurntSushi/ripgrep
[elm-json]: https://github.com/zwilias/elm-json
[forum-cross]: https://users.rust-lang.org/t/cross-compile-macos-and-ms-windows/38323
[medium-github-action]: https://medium.com/@jondot/building-rust-on-multiple-platforms-using-github-6f3e6f8b8458
