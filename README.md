# elm-test-rs

Attempt at a simpler, faster alternative to the current node test runner for elm-test, but in Rust.

## Expected features

- Same interface as elm-test
- Ability to change the number of processes
- Fix logging issues for machine reports

## Already implemented

- [x] elm-test-rs --version
- [ ] elm-test-rs init
- [ ] elm-test-rs install
- [ ] elm-test-rs
- [ ] elm-test-rs TESTFILES
- [ ] elm-test-rs --compiler /path/to/compiler
- [ ] elm-test-rs --seed integer
- [ ] elm-test-rs --fuzz integer
- [ ] elm-test-rs --report json
- [ ] elm-test-rs --report junit
- [ ] elm-test-rs --watch
- [ ] (new) elm-test-rs --processes integer

## Workflow of a test runner

1. Generate the list of test modules and their file paths.
2. Generate a correct `elm.json` for the to-be-generated new `Main.elm`.
    1. Join project and this test runner `source-directories`.
    2. Generate a correct list of `dependencies` (leverage elm-json).
3. Compile all test files with `elm make --output=/dev/null <all/test/files>`
   such that we know they are correct elm files.
4. Find all tests
    1. Either parse directly the test files.
    2. Or parse the .elmi compilation artifacts (contains only exposed values?).
5. Generate the `Main.elm` with one test concatenating all found exposed tests.
6. Compile this main into a JS file.
7. Compose a Node module "test worker" (JS file) encapsulating the main elm js file
   with the ports and all network communication code to exchange data with the supervisor.
8. Run the supervisor program.
    1. Create a server socket that will listen to connections from test workers.
    2. Spawn node test workers.
    3. Distribute work to the workers.
    4. Gather test results.
    5. Print test results according to the report format.

## Proof of concept TODOs

- [x] `examples/glob.rs` List all test module files. We must be able to expand globs passed as arguments.
- [ ] Read and write a correct `elm.json`. Leverage zwilias/elm-json library for dependencies.
- [ ] Make zwilias/elm-json offline capable by limiting constraints to installed packages.
- [x] `examples/command.rs` Call the elm compiler binary.
- [ ] Parse test files or .elmi files to find all exposed tests.
- [ ] Parse test files to find unexposed tests.
      Might be tricky to avoid false positive due to functions like `describe` that can embed tests.
- [x] `examples/template_elm.rs` Generate a templated `Main.elm` file from a list of tests.
- [x] `examples/template_js.rs` Generate a templated JS file.
- [x] `examples/tcp_server.rs` Create a server tcp socket able to exchange data with client tcp socket.
- [x] `examples/supervisor/` Example worker communication between supervisor, runner and reporter.
- [ ] Convert results into console/json/junit reports.
- [ ] Remove the report option from the elm test worker,
      it should only be concerned by one communication format,
      it's the supervisor work to convert to the appropriate output.
- [ ] Make cross-platform binaries

## Some thoughts

- This new alternative for elm-test should be as simple and lightweight as possible.
- For the CLI, we could use [pico-args][pico-args] (lightweight)
  or [clap][clap] (heavyheight but most used).
- Other useful CLI tools may be available (see https://lib.rs/command-line-interface)
- For the generation of the main elm file, we could use [TinyTemplate][TinyTemplate].
  I opted for [sonro/varj][varj] which is quite minimalist.

[pico-args]: https://github.com/RazrFalcon/pico-args
[clap]: https://github.com/clap-rs/clap
[TinyTemplate]: https://github.com/bheisler/TinyTemplate
[varj]: https://github.com/sonro/varj

## Communication between the Elm (node) process and Rust supervisor

It seems that this will not be trivial.
The node module spawned for the Elm code currently uses
`client = net.createConnection(pipeFilename)`
(CF `templates/after.js`).
According to [node documentation][createConnection],
this initiates an IPC (Inter Process Communication) connection and returns
the new [`net.Socket`][socket].

[createConnection]: https://nodejs.org/api/net.html#net_net_createconnection
[socket]: https://nodejs.org/api/net.html#net_class_net_socket

I've only quickly searched but a platform agnostic IPC socket in Rust
does not seam to be trivial.
Maybe a simple TCP socket connection at any available port is sufficient performance-wise.
CF `examples/tcp-client-server/` where we could replace `server.js`
by a Rust TCP server.

Random links to related articles:

- Norbert de Langen, 2017, [Communicating between NodeJS processes][norbert2017]

[norbert2017]: https://medium.com/@NorbertdeLangen/communicating-between-nodejs-processes-4e68be42b917

## Cross compilation for OSX and Windows

- Look at configuration of [BurntSushi/ripgrep][ripgrep]
- Look at configuration of [zwilias/elm-json][elm-json]
- Discussion on Rust forum: [Cross compile macOS and MS Windows][forum-cross]
- Medium post by Dotan Nahum:
  [Building Rust for Multiple Platforms Using Github Actions][medium-github-action]

[ripgrep]: https://github.com/BurntSushi/ripgrep
[elm-json]: https://github.com/zwilias/elm-json
[forum-cross]: https://users.rust-lang.org/t/cross-compile-macos-and-ms-windows/38323
[medium-github-action]: https://medium.com/@jondot/building-rust-on-multiple-platforms-using-github-6f3e6f8b8458

## Embedding another executable?

Would it be possible to embed another binary at build time inside ours (such as elmi-to-json)
and execute it at runtime?
I don't think it is possible to embed another executable and run it from our binary,
especially with cross-platform support, it seems very unlikely.

- [pyros2097/rust-embed][rust-embed]: Rust Macro which loads files into the rust Binary at compile time

[rust-embed]: https://github.com/pyros2097/rust-embed

## Executing another command

An alternative, if we do not rewrite elmi-to-json,
is to assume a user of elm-test-rs has it installed already on their system.
Therefore, we can execute it as a command.

- `std::process::Command` [documentation][command]

[command]: https://doc.rust-lang.org/std/process/struct.Command.html
