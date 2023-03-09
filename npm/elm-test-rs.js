#!/usr/bin/env node

var child_process = require("child_process");

var binaryPath = require("./binary.js")();

child_process
  .spawn(binaryPath, process.argv.slice(2), { stdio: "inherit" })
  .on("exit", process.exit);
