//! Print message about the non existing --watch option.

/// Print message about the non existing --watch option.
pub fn main() {
    println!("{}", WATCH);
}

const WATCH: &str = r#"
The --watch option does not exist for elm-test-rs contrary to elm-test.
There is no need for it due to the speed at which elm-test-rs generates the tests file.
The different steps of elm-test-rs are the following:

1. Generate Runner.elm containing all the tests to run
2. Generate Reporter.elm containing the reporting code
3. Compile Runner.elm and Reporter.elm
4. Start Node.js supervisor
5. Run the tests

In elm-test, the --watch mode is useful since step (1) might
take a non negligeable amount of time, but even in --watch mode,
step (3) needs to be redone when you modify your code.
In elm-test-rs, steps (1) and (2) is usually faster than just spawning Node.js (4).
On my computer, even on a decently size tests suite such as elm-geometry or elm-css,
steps (1) and (2) take around 10ms.
The only time gain in elm-test-rs would the time to spawn Node.js (4).
For the time being, it is not worth the added complexity.

So we suggest using a dedicated program to watch files such as watchexec
(https://github.com/watchexec/watchexec).
You can combine it very easily with elm-test-rs, for example:

watchexec elm-test-rs
watchexec elm-test-rs -- --workers 1
"#;
