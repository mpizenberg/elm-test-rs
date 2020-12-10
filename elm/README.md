# elm-test-runner

Helper elm package to run elm tests and report results.


## Test runner architecture

There is a main CLI built in Rust in the repository [mpizenberg/elm-test-rs][elm-test-rs].
It is tasked to fiddle with tests files, generate some Elm and JS files and run them.
It ends with the start of a basic NodeJS supervisor program,
supervising runners in workers and a reporter.
Both the runner code and the reporter code are in Elm.
You can find them here, in the modules `ElmTestRunner.Runner` and `ElmTestRunner.Reporter`.
The following diagram summarizes the architecture
and communication channels of the supervisor, reporter and runners.

![architecture diagram][runner-diagram]

[elm-test-rs]: https://github.com/mpizenberg/elm-test-rs
[runner-diagram]: https://mpizenberg.github.io/resources/elm-test-rs/elm-test-runner.png
