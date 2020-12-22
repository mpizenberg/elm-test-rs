const { parentPort } = require("worker_threads");
const { performance } = require("perf_hooks");

// From templates/polyfills.js
{{ polyfills }}

// Capture Debug.log from elm code
let logs = [];
process.stdout.write = (str) => logs.push(str);

// Compiled by elm-test-rs from templates/Runner.elm
const { Elm } = require("./Runner.elm.js");

// Start the Elm app
const flags = { initialSeed: {{ initialSeed }}, fuzzRuns: {{ fuzzRuns }} };
const app = Elm.Runner.init({ flags: flags });

// Record the timing at which we received the last "runTest" message
let startTime;

// Communication from Supervisor to Elm runner via port
parentPort.on("message", (msg) => {
  if (msg.type_ == "askTestsCount") {
    app.ports.askTestsCount.send();
  } else if (msg.type_ == "runTest") {
    startTime = performance.now();
    app.ports.receiveRunTest.send(msg.id);
  } else {
    console.error("Invalid supervisor msg.type_:", msg.type_);
  }
});

// Communication from Elm runner to Supervisor via port
// Subscribe to outgoing Elm ports defined in templates/Runner.elm
app.ports.sendResult.subscribe((msg) => {
  msg.type_ = "testResult";
  msg.duration = performance.now() - startTime;
  msg.logs = logs;
  parentPort.postMessage(msg);
  logs = [];
});
app.ports.sendTestsCount.subscribe((msg) => {
  msg.type_ = "testsCount";
  msg.logs = logs;
  parentPort.postMessage(msg);
  logs = [];
});
