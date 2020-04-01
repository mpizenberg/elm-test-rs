# elm-test-rs

Attempt at a simpler, faster alternative to the current node test runner for elm-test, but in Rust.

## Expected features

- Same interface as elm-test
- Ability to change the number of processes
- Fix logging issues for machine reports

## Already implemented

- [ ] elm-test-rs --version
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

## Some thoughts

- This new alternative for elm-test should be as simple and lightweight as possible.
- For the CLI, we could use [pico-args][pico-args] (lightweight)
  or [clap][clap] (heavyheight but most used).
- Other useful CLI tools may be available (see https://lib.rs/command-line-interface)
- For the generation of the main elm file, we could use [TinyTemplate][TinyTemplate].

[pico-args]: https://github.com/RazrFalcon/pico-args
[clap]: https://github.com/clap-rs/clap
[TinyTemplate]: https://github.com/bheisler/TinyTemplate

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

- Discussion on Rust forum: [Cross compile macOS and MS Windows][forum-cross]
- Medium post by Dotan Nahum:
  [Building Rust for Multiple Platforms Using Github Actions][medium-github-action]

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
