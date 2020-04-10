const { Elm } = require("./Reporter.js");

const possibleReporters = ["console", "json", "junit"];
const possibleOutcomes = ["passed", "todo", "failed"];
const maxTests = 10;

// Generate a random number of tests
const randomInt = (maxInt) => Math.floor(Math.random() * maxInt);
const nbTests = Math.max(1, randomInt(maxTests));

// Initialize the Elm app
const mode = possibleReporters[randomInt(possibleReporters.length)];
const flags = { mode: mode, nbTests: nbTests };
const app = Elm.Reporter.init({ flags: flags });

// Subscribe to outgoing Elm ports
app.ports.signalFinished.subscribe((str) => console.log(str));
app.ports.stdout.subscribe((str) => process.stdout.write(str));

// Generate the random test results
for (let i = 0; i < nbTests; i++) {
  const labels = ["Test", i.toString()];
  const outcome = possibleOutcomes[randomInt(possibleOutcomes.length)];
  const duration = Math.random();
  const result = { labels: labels, outcome: outcome, duration: duration };
  app.ports.incomingResult.send(result);
}
