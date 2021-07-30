// From templates/polyfills.js
{{ polyfills }}

// Compiled by elm-test-rs from templates/Runner.elm
import { Elm } from "./Runner.elm.mjs";

// Capture Debug.log from elm code
// which has been kernel-switched to "console.elmlog"
let logs = [];
console.elmlog = (str) => logs.push(str);

// Start the Elm app
const flags = { initialSeed: {{ initialSeed }}, fuzzRuns: {{ fuzzRuns }}, filter: {{ filter }} };
const app = Elm.Runner.init({ flags: flags });

// Record the timing at which we received the last "runTest" message
let startTime;

// Communication from Supervisor to Elm runner via port
self.onmessage((msg) => {
  if (msg.data.type_ == "askTestsCount") {
    app.ports.askTestsCount.send();
  } else if (msg.data.type_ == "runTest") {
    startTime = performance.now();
    app.ports.receiveRunTest.send(msg.data.id);
  } else {
    console.error("Invalid supervisor msg.type_:", msg.data.type_);
  }
});

// Communication from Elm runner to Supervisor via port
// Subscribe to outgoing Elm ports defined in templates/Runner.elm
app.ports.sendResult.subscribe((msg) => {
  msg.type_ = "testResult";
  msg.duration = performance.now() - startTime;
  msg.logs = logs;
  self.postMessage(msg);
  logs = [];
});
app.ports.sendTestsCount.subscribe((msg) => {
  msg.type_ = "testsCount";
  msg.logs = logs;
  self.postMessage(msg);
  logs = [];
});
