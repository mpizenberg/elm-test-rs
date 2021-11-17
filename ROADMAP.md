# Roadmap

The elm-test-rs test runner is mainly developed on my free time, free time that is highly disputed with other side projects and time spent with friends and family.
As such, I do not have a clear roadmap or any deadline in mind since I cannot predict when I'll have both the time and desire to make progress here.
However I can outline the priorities for this project.

1. Be usable with the latest version of Elm. This means when a new version of elm is released, which becomes incompatible, my priority will be to update elm-test-rs as soon as possible.
2. Be reactive to answer user requests and bugs. If those are easy fixes, do it or outline the fix idea for a PR. If those are not easy fixes, either try explaining why they would take some time, or why they won't be done.
3. Try some cool new ideas that would improve user experience and ease of use.

Something I'd also love is to have enough community members familiar with the code base of elm-test-rs, that I could eventually fully delegate the project to others I trust.
As such, all contributions, be it bug reports, ideas, and of course pull requests, are welcome.
Everyone should feel welcome in this project, and I shall change anything going against this.

## Cool new ideas

### Language server integration

In order to improve user experience in IDEs, we've started discussions with the language server maintainers to try an integration of elm-test-rs.
There would be some speed advantages to check that the tests compile since elm-test-rs does not start a Node runtime before actually running the tests.
Not much gains regarding actual tests running however since this takes roughly the same time.

### Web runner and report

Currently, elm-test-rs supports both Node and Deno runtimes.
In the future, I'd like to also have a Web runtime, which would enable running Elm tests without even needing an installation of Node or Deno, just have a compatible Web Browser.
The actual JS code for the runner should be very similar to Deno, so there would not be a need for a ton of work there I think.
However, there would be quite some work around the communication between elm-test-rs and the browser, on the reports, and on the watch mode.
