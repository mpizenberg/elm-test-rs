process.chdir(__dirname);

// From templates/polyfills.js
{{ polyfills }}

const { Worker } = require("worker_threads");
const readline = require("readline");
const EventEmitter = require("events");

// Global variables
let nbTests, doneTests, todoTests;
let reporter;
let runners = [];
let working = false;
let nb_workers = {{ nb_workers }};
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
reporter.ports.signalFinished.subscribe((code) => {
  runners.forEach((runner) => runner.terminate());
  working = false;
  supervisorEvent.emit("finishedWork");
  process.exit(code);
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
  runners[0] = new Worker(runnerFile, { stdout: true });//, stderr: true });
  runners[0].on("message", (msg) =>
    handleRunnerMsg(runners[0], 0, runnerFile, msg)
  );
  runners[0].on("online", () =>
    runners[0].postMessage({ type_: "askNbTests" })
  );
}

// Handle a test result
function handleRunnerMsg(runner, runnerId, runnerFile, msg) {
  if (msg.type_ == "nbTests") {
    setupWithNbTests(runnerFile, msg.nbTests);
  } else if (msg.type_ == "result") {
    handleResult(runner, msg.id, msg.result);
  } else if (msg.type_ == "logs") {
    reporter.ports.incomingLogs.send({ runnerId: runnerId, logs: msg.logs });
  } else {
    console.error("Invalid runner msg.type_:", msg.type_);
  }
}

// Reset supervisor tests count and reporter
// Start work on all runners
function setupWithNbTests(runnerFile, nb) {
  // Reset supervisor tests
  nbTests = nb;
  doneTests = Array(nb).fill(false);
  todoTests = Array(nb)
    .fill(0)
    .map((_, id) => id)
    .reverse();
  // Reset reporter
  reporter.ports.restart.send(nb);
  // Send first runner job
  runners[0].postMessage({ type_: "runTest", id: todoTests.pop() });

  // Create and send work to all other workers.
  let max_workers = Math.min(nb_workers, nbTests);
  for (let i = 1; i < max_workers; i++) {
    runners[i] = new Worker(runnerFile, { stdout: true });//, stderr: true });
    runners[i].on("message", (msg) =>
      handleRunnerMsg(runners[i], i, runnerFile, msg)
    );
    runners[i].on("online", () =>
      runners[i].postMessage({ type_: "runTest", id: todoTests.pop() })
    );
  }
}

// Update supervisor tests, transfer result to reporter and ask to run another test
function handleResult(runner, id, result) {
  doneTests[id] = true;
  const nextTest = todoTests.pop();
  if (nextTest != undefined) {
    runner.postMessage({ type_: "runTest", id: nextTest });
  }
  reporter.ports.incomingResult.send(result);
}
