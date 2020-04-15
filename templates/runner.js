const { parentPort } = require("worker_threads");

// From templates/polyfills.js
{{ polyfills }}

const Elm = (function (module) {
  // Compiled by elm-test-rs from templates/Runner.elm
  {{ compiled_elm }}
  return this.Elm;
})({});

// Start the Elm app
const flags = { initialSeed: {{ initialSeed }}, fuzzRuns: {{ fuzzRuns }} };
const app = Elm.Runner.init({ flags: flags });

// Communication from Supervisor to Elm runner via port
parentPort.on("message", (msg) => {
  if (msg.type_ == "askNbTests") {
    app.ports.askNbTests.send(null);
  } else if (msg.type_ == "runTest") {
    app.ports.receiveRunTest.send(msg.id);
  } else {
    console.error("Invalid supervisor msg.type_:", msg.type_);
  }
});

// Communication from Elm runner to Supervisor via port
// Subscribe to outgoing Elm ports defined in elm/src/ElmTestRs/Test/Runner.elm
app.ports.sendResult.subscribe((msg) => parentPort.postMessage(msg));
app.ports.sendNbTests.subscribe((msg) => parentPort.postMessage(msg));
