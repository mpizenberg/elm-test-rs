# Creation of a new release

This is taking the 3.0.0 release as an example.

## GitHub stuff

- Update the release date in the changelog and push
- Check that the tests are passing
- `git tag -a v3.0 -m "v3.0: Support for the new elm-explorations/test v2"`
- (Optional) cryptographically sign the tag
- Push release to github: git push --follow-tags
- On GitHub, create a release from that tag
- Include the prebuilt executables from the CI in the GitHub release

The CI builds executables for each supported platform.
Simply download and unzip all archives (except Windows one) to be left with the `tar.gz` archive.
Then rename it to follow the already taken convention:

- Linux ARM 32bit: `elm-test-rs_linux-arm-32.tar.gz`
- Linux ARM 64bit: `elm-test-rs_linux-arm-64.tar.gz`
- Linux Intel x86 64bit: `elm-test-rs_linux.tar.gz `
- Apple MacOS ARM 64bit: `elm-test-rs_macos-arm.tar.gz `
- Apple MacOS Intel x86 64bit: `elm-test-rs_macos.tar.gz`
- Windows Intel x86 64bit: `elm-test-rs_windows.zip`

## NPM stuff

Repeat this for all the binary packages in `npm/packages/`.
This uses `npm/packages/elm-test-rs-darwin-x64` as an example.

1. Go to the folder: `cd npm/packages/elm-test-rs-darwin-x64`
2. Copy the appropriate binary to `./elm-test-rs`. For Windows: `./elm-test-rs.exe`
3. Double-check that you put the right binary in the right package: `file elm-test-rs`
4. Double-check that the file is executable: `ls -l elm-test-rs`
5. In `package.json` of the binary package, bump the version for example to `"3.0.0-0`".
6. In `package.json` of the main npm package, update `"optionalDependencies"` to point to the bumped version.
  For example: `"@mpizenberg/elm-test-rs-darwin-x64": "3.0.0-0"`.
  Note: Pin the versions of the binary packages exactly, no version ranges.
  This means that installing `elm-test-rs@3.0.0-0` installs the exact same bytes in two years as today.
7. Publish the package: `npm publish --access=public`
  `--access=public` is needed because scoped packages are private by default.

Then publish the main npm package by running `npm publish` in the `npm/` folder.

## Crates.io stuff

Maybe one day, with cargo-binstall?

## Community stuff

Talk about the awesome new features of the new release online.
