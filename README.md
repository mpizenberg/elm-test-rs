# elm-test-rs

Fast and portable executable to run your Elm tests.

## Install

To install elm-test-rs **globally**, simply download the executable for your system
from the [latest release](https://github.com/mpizenberg/elm-test-rs/releases),
and put it in a directory in your `PATH` environment variable
so that you can call `elm-test-rs` from anywhere.

To install elm-test-rs **locally** per project,
add elm-test-rs in your `elm-tooling.json` config file
and use [`elm-tooling install`][elm-tooling].
In such case, you'll have to run it via npx: `npx elm-test-rs`.

To install elm-test-rs **in your CI**,
as well as elm and other tools, use this [GitHub action][action].

[elm-tooling]: https://elm-tooling.github.io/elm-tooling-cli/
[action]: https://github.com/mpizenberg/elm-tooling-action

## Usage

Use `elm-test-rs init` to setup tests dependencies and create `tests/Tests.elm`.

```shell
> elm-test-rs init
The file tests/Tests.elm was created
```

And simply use `elm-test-rs` to compile and run all your tests.

```shell
> elm-test-rs

Running 1 tests. To reproduce these results later,
run elm-test-rs with --seed 597517184 and --fuzz 100

◦ TODO: Implement the first test. See https://package.elm-lang.org/packages/elm-explorations/test/latest for how to do this!

TEST RUN INCOMPLETE because there is 1 TODO remaining

Duration: 1 ms
Passed:   0
Failed:   0
Todo:     1
```

Information on how to write tests is available at https://github.com/elm-explorations/test/.

## New features compared to elm-test

### Capturing `Debug.log` outputs

With elm-test-rs, calls to `Debug.log` are captured
and displayed in context with the associated failing test.
Let's say we have the following source file.

```elm
module Question exposing (answer)

answer : String -> Int
answer question =
    let
        _ =
            Debug.log "The question was" question
    in
    if question == "What is the Answer to the Ultimate Question of Life, The Universe, and Everything?" then
        43

    else
        0
```

And we have the following tests file.

```elm
module Tests exposing (..)

import Expect
import Question
import Test exposing (Test)

suite : Test
suite =
    Test.describe "Question"
        [ Test.test "answer" <|
            \_ ->
                Question.answer "What is the Answer to the Ultimate Question of Life, The Universe, and Everything?"
                    |> Expect.equal 42
        ]
```

Then `elm-test-rs` will give you the following output.

```txt
Running 1 tests. To reproduce these results later,
run elm-test-rs with --seed 2433154680 and --fuzz 100

↓ Question
✗ answer

    43
    ╷
    │ Expect.equal
    ╵
    42

    with debug logs:

The question was: "What is the Answer to the Ultimate Question of Life, The Universe, and Everything?"


TEST RUN FAILED

Duration: 2 ms
Passed:   0
Failed:   1
```

There are still improvements to be made since fuzz tests will report
all their logs instead of just the simplest one,
but this is already super useful for unit tests.

### Deno runtime

By default, `elm-test-rs` runs the tests with Node.
It is possible however to run the tests with [Deno][deno] instead of Node with `elm-test-rs --deno`.
This makes testing more accessible in places where Node is tedious to install.

[deno]: https://deno.land/

### Verbosity

By default, elm-test-rs just prints to stdout the output of the tests runner,
which are dependent on the `--report` option chosen (defaults to console report).
But if you are interested in gaining more insight on what is happening inside,
you can add a verbosity level to the command.

- `elm-test-rs -v`: Slightly verbose. This will print to stderr some additional info
  like the version of elm-test-rs being used, or the total amount of time
  spent in the Node process spawned to run the tests.
- `elm-test-rs -vv`: Very verbose. This will print to stderr all the steps
  leading to running the tests.
- `elm-test-rs -vvv`: Debug verbose. This will print some additional info to stderr
  that might be useful to report in an issue if you encounter a crash.

Currently, the verbosity level only impacts the stderr output generated
by elm-test-rs before and after running the tests.
It does not change the stdout output of the tests runner itself.
We could also change verbosity of the console reporter in the future,
but that is not planned as a priority.

### Choose newest or oldest package dependencies

For packages authors, it is sometimes hard to check that a dependency
lower bound is actually working with your package when `elm-test`
always installs the newest compatible version of a given package to run the tests.
With `elm-test-rs -vv --dependencies newest` in "very verbose" mode, it will tell you
which version of each package was used to run the tests.
For `mdgriffith/elm-ui` for example, it will give the following.

```js
{
  "direct": {
    "elm/core": "1.0.5",
    "elm/html": "1.0.0",
    "elm/json": "1.1.3",
    "elm/virtual-dom": "1.0.2",
    "elm-explorations/test": "1.2.2",
    "mpizenberg/elm-test-runner": "4.0.3"
  },
  "indirect": {
    "elm/random": "1.0.0",
    "elm/time": "1.0.0"
  }
}
```

While if you run `elm-test-rs -vv --dependencies oldest`, you will get those.

```js
{
  "direct": {
    "elm/core": "1.0.0",
    "elm/html": "1.0.0",
    "elm/json": "1.0.0",
    "elm/virtual-dom": "1.0.0",
    "elm-explorations/test": "1.2.2",
    "mpizenberg/elm-test-runner": "4.0.3"
  },
  "indirect": {
    "elm/random": "1.0.0",
    "elm/time": "1.0.0"
  }
}
```

### Offline mode

By default, elm-test-rs will try using the packages already installed
on your machine, but if there is something missing, it will connect
to the package website to check existing versions of packages that could be used.
If you want, you can prevent that second phase from happening, making it crash instead.
To do that, just add `--offline` to the elm-test-rs command.

Note that the `--offline` and `--dependencies` flags are incompatible with each other,
as you generally can't know which are the oldest or newest existing packages
without asking the package site which version exist.

### Other useful features

- `--workers N` lets you specify the amount of worker threads spawn to run the tests.
  Sometimes when you processor reports more threads than cores, like 2 cores and 4 threads,
  you actually get slightly better performance by specifying `--workers 2` instead
  of its default that will be 4.
  You might also want to limit it to 1 worker for some reasons.
- `--filter substring` lets you only run tests whose description contain
  the given string passed as argument.
  This can be more convenient than to add `Test.only` in your tests.
  It also makes it easy to run a group of tests identifiable by their descriptions.

Check out the command help with `elm-test-rs --help` to know more about all its features.

## Differences with elm-test

Both elm-test and elm-test-rs are very similar,
especially since version 0.19.1-revision5 of elm-test.
However, there are still few differences.
Some are small differences:

- the `console` output isn't exactly the same
- the `install` command isn't implemented yet (use elm-json for that)

Some might make your tests crash with elm-test-rs.

### No automatic module description

With elm-test, the module name is automatically prepended to descriptions
of all its tests, meaning you can have the same description for tests
in different modules.
With elm-test-rs, there is no such thing, your descriptions are entirely explicit
and left untouched, so you cannot compile multiple test modules with the same
description tests inside or you will get a "duplicate test name" error.
To understand the reasons of this choice,
please have a look at that [GitHub issue][duplicate].

[duplicate]: https://github.com/rtfeldman/node-test-runner/issues/493

The easiest way to fix such "duplicate test name" error
is to create a new `Test.describe` level for the corresponding modules, tranforming

```elm
TestModule exposing (a, b, c)
```

into

```elm
TestModule exposing (tests)

tests = describe "TestModule" [ a, b, c ]
```

### Globs are treated slightly differently

Whith elm-test, globs support directories so you can call `elm-test tests/` and all elm files
within the `tests/` directory will be used.
With elm-test-rs the arguments must be elm files,
so you would call `elm-test-rs tests/**/*.elm` instead.

## Minimum supported version

- Elm 0.19.1
- Node 10.5

## Design goals

In addition to new useful features,
elm-test-rs aims to be easy to maintain and to extend.
For these reasons, the core design goals are for the code to be

- as simple and lightweight as reasonably possible,
- modular,
- well documented.

## Code architecture

The code of this project is split in three parts.

 1. The CLI, a rust application that generates all the needed JS and Elm files to run tests.
 2. The supervisor, a small Node JS script
    (roughly 100 lines, no dependency other than Node itself)
    tasked to spawn runners (Elm), start a reporter (Elm)
    and transfer tests results from the runners to the reporter.
 3. An Elm package [mpizenberg/elm-test-runner][elm-test-runner]
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
The reporter is just loaded from its compiled elm code by the supervisor.
Communication between the Elm and JS parts are done through ports, as usual.

The Elm package containing the code for runners and reporters
is [mpizenberg/elm-test-runner][elm-test-runner].

![architecture diagram][diagram]

[diagram]: https://mpizenberg.github.io/resources/elm-test-rs/elm-test-rs.png
[elm-test-runner]: https://github.com/mpizenberg/elm-test-runner


## Contributing

Contributions are very welcome.
This repository holds a submodule so make sure to clone it recursively.

```sh
git clone --recursive ...
```

To build the `elm-test-rs` binary, [install Rust][install-rust] and run the command:

```sh
cargo build --release
```

The executable will be located at `target/release/elm-test-rs`.

This project also uses [rust format][rustfmt] and [clippy][clippy]
(with its default options) to enforce good code style.
To install these tools run

```bash
rustup update
rustup component add clippy rustfmt
```

and then before committing run

```bash
cargo fmt -- --check
touch src/main.rs && cargo clippy
```

PS: clippy is a rapidly evolving tool so if there are lint errors on CI
don't forget to `rustup update`.

[install-rust]: https://www.rust-lang.org/tools/install
[rustfmt]: https://github.com/rust-lang/rustfmt
[clippy]: https://github.com/rust-lang/rust-clippy
