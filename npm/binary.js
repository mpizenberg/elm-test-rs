var fs = require("fs");
var path = require("path");
var package = require("./package.json");

module.exports = function () {
  var platform = `${process.platform}-${process.arch}`;
  var subPackageName = `@mpizenberg/elm-test-rs-${platform}`;

  if (!(subPackageName in package.optionalDependencies)) {
    exitFailure(
      `The elm-test-rs npm package does not support your platform (${platform}).`
    );
  }

  var fileName =
    process.platform === "win32" ? "elm-test-rs.exe" : "elm-test-rs";

  try {
    var subBinaryPath = require.resolve(`${subPackageName}/${fileName}`);
  } catch (error) {
    if (error && error.code === "MODULE_NOT_FOUND") {
      exitFailure(missingSubPackageHelp(subPackageName));
    } else {
      exitFailure(
        `I had trouble requiring the binary package for your platform (${subPackageName}):\n\n${error}`
      );
    }
  }

  // Yarn 2 and later ("Berry") always invokes `node` (regardless of configuration)
  // so we cannot do any optimizations there.
  var isYarnBerry = /\byarn\/(?!1\.)/.test(
    process.env.npm_config_user_agent || ""
  );

  // On Windows, npm always invokes `node` so we cannot do any optimizations there either.
  if (process.platform === "win32" || isYarnBerry) {
    return subBinaryPath;
  }

  var binaryPath = path.resolve(__dirname, package.bin.elm);
  var tmpPath = binaryPath + ".tmp";

  try {
    // Atomically replace the file with a hard link to the binary as an optimization.
    fs.linkSync(subBinaryPath, tmpPath);
    fs.renameSync(tmpPath, binaryPath);
  } catch (error) {
    exitFailure(
      `I had some trouble writing file to disk. It is saying:\n\n${error}`
    );
  }

  return binaryPath;
};

function exitFailure(message) {
  var tag = `v${package.version.replace(/^(\d+\.\d+)\.0$/, "$1")})}`;
  console.error(
    `
-- ERROR -----------------------------------------------------------------------

${message}

NOTE: You can avoid npm entirely by downloading directly from:
https://github.com/mpizenberg/elm-test-rs/releases/tag/${tag}
All this package does is distributing a file from there.

--------------------------------------------------------------------------------
    `.trim()
  );
  process.exit(1);
}

function missingSubPackageHelp(subPackageName) {
  return `
I support your platform, but I could not find the binary package (${subPackageName}) for it!

This can happen if you use the "--omit=optional" (or "--no-optional") npm flag.
The "optionalDependencies" package.json feature is used by elm-test-rs to install the correct
binary executable for your current platform. Remove that flag to use elm-test-rs.

This can also happen if the "node_modules" folder was copied between two operating systems
that need different binaries - including "virtual" operating systems like Docker and WSL.
If so, try installing with npm rather than copying "node_modules".
    `.trim();
}
