const { Worker } = require("worker_threads");

const nbTests = 3;
const doneTests = Array(nbTests).fill(false);
const todoTests = Array(nbTests)
  .fill(0)
  .map((_, id) => id);

// Start reporter worker
const reporter = new Worker("./reporter.js");
reporter.on("message", (msg) => console.log("from reporter:", msg));
reporter.on("online", () => {
  reporter.postMessage({ type: "START", nbTests: nbTests });
});

// Start runner worker and prevent piped stdout and sdterr
const runner = new Worker("./runner.js", { stdout: true, stderr: true });
runner.on("message", handleRunnerMsg);
runner.on("online", () => {
  runner.postMessage({ type: "TEST", id: todoTests.pop() });
});

// Handle result of a test
function handleRunnerMsg(msg) {
  reporter.postMessage(msg);
  doneTests[msg.id] = true;
  // console.log("todoTests:", todoTests);
  const nextTest = todoTests.pop();
  // console.log("nextTest:", nextTest);
  if (nextTest != undefined) {
    runner.postMessage({ type: "TEST", id: nextTest });
  }
}
