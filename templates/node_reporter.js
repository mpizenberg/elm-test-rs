// From templates/polyfills.js
{{ polyfills }}

// Compiled by elm-test-rs from templates/Reporter.elm
const { Elm } = require("./Reporter.elm.js");

// Start the Elm app
const flags = {
  initialSeed: {{ initialSeed }},
  fuzzRuns: {{ fuzzRuns }},
  mode: "{{ reporter }}",
};
const app = Elm.Reporter.init({ flags: flags });

// Pipe the Elm stdout port to stdout
app.ports.stdout.subscribe((str) => process.stdout.write(str));

// Export function to set the callback function used when reports are finished
let finishCallback = () => console.error("finishCallback not defined yet");
app.ports.signalFinished.subscribe((code) => finishCallback(code));
exports.setCallback = (callback) => { finishCallback = callback; };

// Export function to restart the Elm reporter
exports.restart = app.ports.restart.send;

// Export function to send results to the Elm reporter
exports.sendResult = app.ports.incomingResult.send;
