// From templates/polyfills.js
{{ polyfills }}

import { readLine } from "./deno_linereader.mjs";
import { Elm } from "./Reporter.elm.js";

// Global variables
let testsCount, todoTests;
let reporter;
let runners = [];
let working = false;
let workersCount = {{ workersCount }};
let startWorkCallback = function(){};
const verbosity = {{ verbosity }};

// Create a long lived reporter worker
const flags = {
  initialSeed: {{ initialSeed }},
  fuzzRuns: {{ fuzzRuns }},
  mode: "{{ reporter }}",
  globs: {{ globs }},
  paths: {{ paths }},
};
reporter = Elm.Reporter.init({ flags: flags });

// Pipe the Elm stdout port to stdout
reporter.ports.stdout.subscribe(
  (str) => Deno.writeAll(Deno.stdout, new TextEncoder().encode(str))
);

// When the reporter has finished clean runners
reporter.ports.signalFinished.subscribe(async ({ exitCode, testsCount }) => {
  runners.map((runner) => runner.terminate());
  working = false;
  startWorkCallback();
  if (verbosity >= 1) {
    console.warn("Running duration (since Node.js start):", Math.round(performance.now()), "ms\n");
  }
  Deno.exit(exitCode);
});

// When receiving a CLI message, start test workers
// The message is a string containing "/path/to/node_runner.js"
for await (let runnerFile of await readLine(Deno.stdin)) {
  runnerFile = "file:" + runnerFile;
  working ? registerWork(runnerFile) : startWork(runnerFile);
}

function registerWork(runnerFile) {
  startWorkCallback = () => startWork(runnerFile);
}

function startWork(runnerFile) {
  startWorkCallback = function(){};
  working = true;
  // Start first runner worker
  runners[0] = new Worker(new URL(runnerFile, import.meta.url).href, { type: "module" });
  runners[0].onmessage = (msg) => handleRunnerMsg(runners[0], runnerFile, msg.data);
  runners[0].postMessage({ type_: "askTestsCount" });
}

function stderrLog(str) {
    Deno.writeAllSync(Deno.stderr, new TextEncoder().encode(str));
}

// Handle a test result
function handleRunnerMsg(runner, runnerFile, msg) {
  if (msg.type_ == "testsCount") {
    if (msg.logs.length > 0) {
      console.warn("Debug logs captured when setting up tests: -----------\n");
      msg.logs.forEach(stderrLog);
      console.warn("\n------------------------------------------------------\n");
    }
    setupWithTestsCount(runnerFile, msg);
  } else if (msg.type_ == "testResult") {
    dispatchWork(runner, todoTests.pop());
    reporter.ports.incomingResult.send(msg);
  } else {
    console.error("Invalid runner msg.type_:", msg.type_);
  }
}

// Reset supervisor tests count and reporter
// Start work on all runners
function setupWithTestsCount(runnerFile, msg) {
  // Reset supervisor tests
  testsCount = msg.testsCount;
  todoTests = Array(testsCount)
    .fill(0)
    .map((_, id) => id)
    .reverse();

  // Reset reporter
  reporter.ports.restart.send(msg);

  // Send first runner job
  if (testsCount == 0) {
    console.error("No exposed values of type Test was found. Did you forget to expose them?");
    return;
  } else {
    runners[0].postMessage({ type_: "runTest", id: todoTests.pop() });
  }

  // Create and send work to all other workers.
  let max_workers = Math.min(workersCount, testsCount);
  for (let i = 1; i < max_workers; i++) {
    let runner = new Worker(new URL(runnerFile, import.meta.url).href, { type: "module" });
    runners[i] = runner;
    runner.onmessage = (msg) => handleRunnerMsg(runner, runnerFile, msg.data);
    dispatchWork(runner, todoTests.pop());
  }
}

// Ask runner to run some test.
function dispatchWork(runner, testId) {
  if (testId != undefined) {
    runner.postMessage({ type_: "runTest", id: testId });
  }
}
