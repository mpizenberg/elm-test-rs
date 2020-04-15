// From templates/polyfills.js
{{ polyfills }}

const Elm = (function(module) {
  // Compiled by elm-test-rs from templates/Runner.elm
  {{ compiled_elm }}
  return this.Elm;
})({});

// Start the Elm app
const flags = {
  initialSeed: {{ initialSeed }},
  fuzzRuns: {{ fuzzRuns }},
  mode: "{{ reporter }}",
};
const app = Elm.{{ elmMain }}.init({ flags: flags });

// Pipe the Elm stdout port to stdout
app.ports.stdout.subscribe((str) => process.stdout.write(str));

// Export function to set the callback function when reports are finished
let finishCallback = () => console.error("finishCallback not defined yet");
app.ports.signalFinished.subscribe((_) => finishCallback());
exports.setCallback = (callback) => { finishCallback = callback; };

// Export function to restart the Elm reporter
exports.restart = app.ports.restart.send;

// Export function to send results to Elm
exports.sendResult = app.ports.incomingResult.send;
