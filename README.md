# elm-test-rs

Attempt at a simpler alternative to node-test-runner for elm tests.


## Usage

Just replace `elm-test` by `elm-test-rs`.


## Install

There is no installation yet, you need to build the tool
and add a link to it in a directory in your PATH env variable.
This repository holds a submodule so make sure to

```sh
git clone --recursive ...
```

To build the `elm-test-rs` binary, install Rust and run the command:

```sh
cargo build --release
```

The executable will be located at `target/release/elm-test-rs`.


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
 - [x] timing of runs

Additional features:

 - [x] `--workers` option to choose the number of runner workers
 - [ ] capturing `Debug.log` calls ([example implementation][capture-log])
 - [ ] progess bar ([example implementation][progress-bar])

[watch-mode]: https://github.com/mpizenberg/elm-test-rs/issues/7
[colors]: https://github.com/mpizenberg/elm-test-rs/issues/5
[pretty-printing]: https://github.com/mpizenberg/elm-test-rs/issues/6
[capture-log]: https://github.com/mpizenberg/elm-test-rs/pull/4
[progress-bar]: https://github.com/mpizenberg/elm-test-rs/pull/3


## Behavior Differences

The node-test-runner (elm-test) automatically adds a
`describe "ModuleName" [ yourTests ]` around your tests in a tests module.
With elm-test-rs no such wrapping is done.
You have to add an explicit `describe` if you want or need one.
This may be the case if you have the same tests in different tests modules,
resulting in a "duplicate test name" error.
In such cases, simply change

```elm
TestModule exposing (a, b, c)
```

into

```elm
TestModule exposing (tests)

tests = describe "TestModule" [ a, b, c ]
```


## Code architecture

The code of this project is split in three parts.

 1. The CLI, a rust application that generates all the needed JS and Elm files to run tests.
 2. The supervisor, a small Node JS script
    (roughly 100 lines, no dependency other than Node itself)
    tasked to spawn runners (Elm), start a reporter (Elm)
    and transfer tests results from the runners to the reporter.
 3. An Elm package (pure, no debug logging) [mpizenberg/elm-test-runner][elm-test-runner]
    exposing a main program for a runner and one for a reporter.

Rust was chosen for the first part since it is a very well fitted language
for systemish CLI programs and enables consise, fast and robust programs.
But any other language could replace this since it is completely independent
from the supervisor, runner and reporter code.
Communication between the CLI and supervisor is assumed to go through STDIN and STDOUT
so no need to lose your hair on weird platform-dependent issues
with inter-process-communication (IPC) going through named pipes.
The CLI program, if asked to run the tests, performs the following actions.

 1. Generate the list of test modules and their file paths.
 1. Generate a correct `elm.json` for the to-be-generated `Runner.elm`.
 1. Find all tests.
 1. Generate `Runner.elm` with a master test concatenating all found exposed tests.
 1. Compile it into a JS file wrapped into a Node worker module.
 1. Compile `Reporter.elm` into a Node module.
 1. Generate and start the Node supervisor program.

To find all tests, we perform a small trick, depending on kernel code (compiled elm code to JS).
First we parse all the tests modules to extract all potential `Test` exposed values.
Then in the template file `Runner.elm` we embed code shaped like this (but not exactly).

```elm
check : a -> Maybe Test
check = ...

main : Program Flags Model Msg
main =
    [ {{ potential_tests }} ]
        |> List.filterMap check
        |> Test.concat
        |> ...
```

This template file gets compiled into a JavaScript file `Runner.elm.js`,
on which we perform the aforementioned kernel patch.
The patch consists in modifying all variants constructors of the `Test` type
to embed a marker, and modifying the `check` function to look for that marker.

Once all the JavaScript code has been generated, it is time to start
the supervisor Node file, which will organize tests runners.
The supervisor and the runners communicate through child and parent worker messages.
The reporter is just loaded as a Node module by the supervisor.
Communication between the Elm and JS parts are done through ports, as usual.
More details about the supervisor, runner and reporter parts are available
in [mpizenberg/elm-test-runner][elm-test-runner].

![architecture diagram][diagram]

[diagram]: https://mpizenberg.github.io/resources/elm-test-rs/elm-test-rs.png
[elm-test-runner]: https://github.com/mpizenberg/elm-test-runner


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
cargo fmt -- --check
touch Cargo.toml && cargo clippy
```

PS: clippy is a rapidly evolving tool so if there are lint errors on CI
don't forget to `rustup update`.

[rustfmt]: https://github.com/rust-lang/rustfmt
[clippy]: https://github.com/rust-lang/rust-clippy
