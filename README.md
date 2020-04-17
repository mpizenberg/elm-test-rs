# elm-test-rs

Attempt at a simpler alternative to node-test-runner for elm tests.


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

 - [ ] `--watch` mode
 - [ ] colors and pretty-printing of diffs
 - [ ] timing of runs

Additional features:

 - [x] `--workers` option to choose the number of runner workers
 - [ ] capturing `Debug.log` calls


## Code architecture

The code of this project is split in three parts.

 1. The CLI, a rust application that generates all the needed JS and Elm files to run tests.
 2. The supervisor, a small Node JS script
    (less than 100 lines, no dependency other than Node itself)
    tasked to spawn runners (Elm), start a reporter (Elm)
    and transfer tests results from the runners to the reporter.
 3. An Elm package (pure, no debug logging) [mpizenberg/elm-test-runner][elm-test-runner]
    exposing a main program for a runner and one for a reporter.

The supervisor and the runners communicate through child and parent worker messages.
The reporter is just loaded as a Node module by the supervisor.
Communication between the Elm and JS parts are done through ports, as usual.
More details about the supervisor, runner and reporter parts are available
in [mpizenberg/elm-test-runner][elm-test-runner].

[elm-test-runner]: https://github.com/mpizenberg/elm-test-runner

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


## Shortcuts and improvements

As this is still a proof of concept, I cut a few corners to get things working.
For example, the generation of the `elm.json` for the tests uses directly
zwilias/elm-json as a binary, and the detection of exposed tests is
done with stoeffel/elmi-to-json as in elm-test.

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


## Embedding template files in the executable?

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
