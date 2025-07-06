# Creation of a new release

This is taking the 3.0.0 release as an example.

## GitHub stuff

- Update the release date in the changelog and push
- Check that the tests are passing
- Update all the NPM package versions inside the npm/ directory
- `git tag -a v3.0.0 -m "v3.0.0: Support for the new elm-explorations/test v2"`
- (Optional) cryptographically sign the tag
- Push release to github: git push --tags
- TODO: edit the draft release notes and then publish the release
- TODO: remove the steps below
- On GitHub, create a release from that tag
- Include the prebuilt executables from the CI in the GitHub release

## NPM stuff

There is an npm package inside the npm/ directory.
When creating a new release, or just needing to update the npm installers,
You can simply update the versions of the root package and subpackages.
Then in the CI, download the npm artifact, republish the subpackages that need republishing,
then republish the root package.

```sh
# --access=public` is needed for scoped packages which are private by default
# Call this inside each subpackage that needs republishing
npm publish --access=public
```

## Crates.io stuff

Maybe one day, with cargo-binstall?

## Community stuff

Talk about the awesome new features of the new release online.
