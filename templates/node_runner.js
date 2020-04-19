const { parentPort } = require("worker_threads");

// From templates/polyfills.js
{{ polyfills }}

// Compiled by elm-test-rs from templates/Runner.elm
const { Elm } = require("./Runner.elm.js");

// Start the Elm app
const flags = { initialSeed: {{ initialSeed }}, fuzzRuns: {{ fuzzRuns }} };
const app = Elm.Runner.init({ flags: flags });

// Redirect console.log to parent port instead
console.log = (str) => {
  parentPort.postMessage({ type_: "logs", logs: str + "\n" });
};

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
// Subscribe to outgoing Elm ports defined in templates/Runner.elm
app.ports.sendResult.subscribe((msg) => parentPort.postMessage(msg));
app.ports.sendNbTests.subscribe((msg) => parentPort.postMessage(msg));
