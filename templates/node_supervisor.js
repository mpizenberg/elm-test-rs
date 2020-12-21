process.chdir(__dirname);

// From templates/polyfills.js
{{ polyfills }}

const { Worker } = require("worker_threads");
const readline = require("readline");
const EventEmitter = require("events");
const { performance } = require("perf_hooks");

// Global variables
let testsCount, todoTests;
let reporter;
let runners = [];
let working = false;
let workersCount = {{ workersCount }};
const supervisorEvent = new EventEmitter();

// Create a long lived reporter worker
const { Elm } = require("./Reporter.elm.js");
const flags = {
  initialSeed: {{ initialSeed }},
  fuzzRuns: {{ fuzzRuns }},
  mode: "{{ reporter }}",
};
reporter = Elm.Reporter.init({ flags: flags });

// Pipe the Elm stdout port to stdout
reporter.ports.stdout.subscribe((str) => process.stdout.write(str));

// When the reporter has finished clean runners
reporter.ports.signalFinished.subscribe(({ exitCode, testsCount }) => {
  if (testsCount == 0) {
    process.stdout.write("There isn't any test, start with: elm-test-rs init\n");
  }
  runners.forEach((runner) => runner.terminate());
  working = false;
  supervisorEvent.emit("finishedWork");
  console.error("Running duration (since Node.js start):", Math.round(performance.now()), "ms\n");
  process.exit(exitCode);
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
  // Start first runner worker and prevent piped stdout and sdterr
  runners[0] = new Worker(runnerFile); //, { stdout: true, stderr: true });
  runners[0].on("message", (msg) =>
    handleRunnerMsg(runners[0], runnerFile, msg)
  );
  runners[0].on("online", () =>
    runners[0].postMessage({ type_: "askTestsCount" })
  );
}

// Handle a test result
function handleRunnerMsg(runner, runnerFile, msg) {
  if (msg.type_ == "testsCount") {
    setupWithTestsCount(runnerFile, msg.testsCount);
  } else if (msg.type_ == "result") {
    dispatchWork(runner, todoTests.pop());
    reporter.ports.incomingResult.send(msg);
  } else {
    console.error("Invalid runner msg.type_:", msg.type_);
  }
}

// Reset supervisor tests count and reporter
// Start work on all runners
function setupWithTestsCount(runnerFile, count) {
  // Reset supervisor tests
  testsCount = count;
  todoTests = Array(count)
    .fill(0)
    .map((_, id) => id)
    .reverse();
  // Reset reporter
  reporter.ports.restart.send(count);

  // Custom handling in the case of no test
  if (testsCount == 0) {
    return;
  }

  // Send first runner job
  runners[0].postMessage({ type_: "runTest", id: todoTests.pop() });

  // Create and send work to all other workers.
  let max_workers = Math.min(workersCount, testsCount);
  for (let i = 1; i < max_workers; i++) {
    let runner = new Worker(runnerFile); //, { stdout: true, stderr: true });
    runners[i] = runner;
    runner.on("message", (msg) =>
      handleRunnerMsg(runner, runnerFile, msg)
    );
    runner.on("online", () => dispatchWork(runner, todoTests.pop()));
  }
}

// Ask runner to run some test.
function dispatchWork(runner, testId) {
  if (testId != undefined) {
    runner.postMessage({ type_: "runTest", id: testId });
  }
}
