const { parentPort } = require("worker_threads");

let tests = { nbTests: 0, doneTests: 0 };

parentPort.on("message", (msg) => {
  if (msg.type == "START") {
    tests.nbTests = msg.nbTests;
    tests.doneTests = 0;
    console.log("reporter: Starting with", tests.nbTests, "tests");
  } else if (msg.type == "TEST_RESULT") {
    tests.doneTests += 1;
    console.log(
      "reporter: Test",
      msg.id,
      msg.success ? "is successful" : "failed"
    );
    if (tests.doneTests == tests.nbTests) {
      console.log("reporter: Finished all", tests.nbTests, "tests");
      parentPort.postMessage({ type: "FINISHED" });
    }
  }
});
