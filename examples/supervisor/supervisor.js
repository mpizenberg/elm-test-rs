const { Worker } = require("worker_threads");
const readline = require("readline");
const EventEmitter = require("events");

// Global variables
let nbTests, doneTests, todoTests;
let reporter, runner;
let working = false;
const supervisorEvent = new EventEmitter();

// Create a long lived reporter worker
reporter = new Worker("./reporter.js");

// When the reporter has finished clean runners
reporter.on("message", (_) => {
  runner.terminate();
  working = false;
  supervisorEvent.emit("finishedWork");
});

// When receiving a CLI message, start test workers
const rl = readline.createInterface({ input: process.stdin });
rl.on("line", (stringConfig) => {
  const config = JSON.parse(stringConfig);
  working ? registerWork(config) : startWork(config);
});

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
  reporter.postMessage({ type: "START", nbTests: nbTests });

  // Start runner worker and prevent piped stdout and sdterr
  runner = new Worker(config.runner, { stdout: true, stderr: true });
  runner.on("message", handleRunnerMsg);
  runner.on("online", () => {
    runner.postMessage({ type: "TEST", id: todoTests.pop() });
  });
}

// Handle result of a test
function handleRunnerMsg(msg) {
  reporter.postMessage(msg);
  doneTests[msg.id] = true;
  const nextTest = todoTests.pop();
  if (nextTest != undefined) {
    runner.postMessage({ type: "TEST", id: nextTest });
  }
}
