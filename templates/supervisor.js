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
// The message is a string containing "/path/to/node_runner.js"
const rl = readline.createInterface({ input: process.stdin });
rl.on("line", (runnerFile) => {
  working ? registerWork(runnerFile) : startWork(runnerFile);
});

function registerWork(runnerFile) {
  supervisorEvent.removeAllListeners(["finishedWork"]);
  supervisorEvent.once("finishedWork", () => startWork(runnerFile));
}

function startWork(runnerFile) {
  working = true;
  // Start runner worker and prevent piped stdout and sdterr
  runner = new Worker(runnerFile); //, { stdout: true, stderr: true });
  runner.on("message", handleRunnerMsg);
  runner.on("online", () => runner.postMessage({ type_: "askNbTests" }));
}

// Handle a test result
function handleRunnerMsg(msg) {
  if (msg.type_ == "nbTests") {
    setupWithNbTests(msg.nbTests);
  } else if (msg.type_ == "result") {
    handleResult(msg.id, msg.result);
  } else {
    console.error("Invalid runner msg.type_:", msg.type_);
  }
}

// Reset supervisor tests count and reporter
// Start work on runner
function setupWithNbTests(nb) {
  // Reset supervisor tests
  nbTests = nb;
  doneTests = Array(nb).fill(false);
  todoTests = Array(nb)
    .fill(0)
    .map((_, id) => id)
    .reverse();
  // Reset reporter
  reporter.restart(nb);
  // Send first runner job
  runner.postMessage({ type_: "runTest", id: todoTests.pop() });
}

// Update supervisor tests, transfer result to reporter and ask to run another test
function handleResult(id, result) {
  doneTests[id] = true;
  const nextTest = todoTests.pop();
  if (nextTest != undefined) {
    runner.postMessage({ type_: "runTest", id: nextTest });
  }
  reporter.sendResult(result);
}
