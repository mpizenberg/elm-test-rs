const { Worker } = require("worker_threads");

// Start supervisor with stdin enabled
const supervisor = new Worker("./supervisor.js", { stdin: true });
supervisor.on("online", () => {
  // Send first round of tests
  supervisor.stdin.write('{"nbTests": 3, "runner": "./runner.js"}');
  supervisor.stdin.write("\n");
  // Send second round (--watch simulation)
  // This round will never be run because we immediately send a third one
  supervisor.stdin.write('{"nbTests": 4, "runner": "./runner.js"}');
  supervisor.stdin.write("\n");
  // Send third round
  supervisor.stdin.write('{"nbTests": 5, "runner": "./runner.js"}');
  supervisor.stdin.write("\n");
  // Send fourth round 1s later
  setTimeout(() => {
    supervisor.stdin.write('{"nbTests": 6, "runner": "./runner.js"}');
    supervisor.stdin.write("\n");
  }, 1000);
});
