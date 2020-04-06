let finishCallback;
let tests = { nbTests: 0, doneTests: 0 };

exports.setCallback = (callback) => {
  finishCallback = callback;
};

exports.start = (nbTests) => {
  tests.nbTests = nbTests;
  tests.doneTests = 0;
  console.log("reporter: Starting with", tests.nbTests, "tests");
};

exports.sendResult = (res) => {
  tests.doneTests += 1;
  console.log(
    "reporter: Test",
    res.id,
    res.success ? "is successful" : "failed"
  );
  if (tests.doneTests == tests.nbTests) {
    console.log("reporter: Finished all", tests.nbTests, "tests");
    finishCallback();
  }
};
