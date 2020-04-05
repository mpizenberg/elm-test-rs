const { parentPort } = require("worker_threads");

parentPort.on("message", (msg) => {
  const success = Math.random() >= 0.5;
  parentPort.postMessage({ type: "TEST_RESULT", id: msg.id, success: success });
});
