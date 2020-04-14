process.chdir(__dirname);

const { Worker } = require("worker_threads");
const readline = require("readline");
const EventEmitter = require("events");

// Global variables
let nbTests, doneTests, todoTests;
let reporter, runner;
let working = false;
const supervisorEvent = new EventEmitter();

// Create a long lived reporter worker
reporter = require("{{ node_reporter }}");

// When the reporter has finished clean runners
reporter.setCallback(() => {
  runner.terminate();
  working = false;
  supervisorEvent.emit("finishedWork");
});

// When receiving a CLI message, start test workers
// The message is a one line JSON of the shape:
// { "nbTests": nbTests, "runner": "/path/to/node_runner.js" }
const rl = readline.createInterface({ input: process.stdin });
rl.on("line", (stringConfig) => {
  const config = JSON.parse(stringConfig);
  working ? registerWork(config) : startWork(config);
});

console.log("Supervisor ready!");

function registerWork(config) {
  console.log("Register work for", config.nbTests, "tests");
  supervisorEvent.removeAllListeners(["finishedWork"]);
  supervisorEvent.once("finishedWork", () => startWork(config));
}

function startWork(config) {
  console.log("Start work for", config.nbTests, "tests");
  working = true;

  // Retrieve test configuration
  nbTests = config.nbTests;
  doneTests = Array(nbTests).fill(false);
  todoTests = Array(nbTests)
    .fill(0)
    .map((_, id) => id);

  // Reset reporter with config
  reporter.restart(nbTests);

  // Start runner worker and prevent piped stdout and sdterr
  console.error("config.runner:", config.runner);
  runner = new Worker(config.runner); //, { stdout: true, stderr: true });
  runner.on("message", handleRunnerResult);
  runner.on("online", () => {
    console.error("runner online");
    runner.postMessage(todoTests.pop());
  });
}

// Handle a test result
function handleRunnerResult(msg) {
  console.error("supervisor: received result", msg.id);
  doneTests[msg.id] = true;
  const nextTest = todoTests.pop();
  if (nextTest != undefined) {
    runner.postMessage(nextTest);
  }
  reporter.sendResult(msg.result);
}
