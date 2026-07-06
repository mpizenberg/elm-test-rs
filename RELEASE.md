# Creation of a new release

Examples use `3.1.0`. Substitute your version throughout.

## 1. Bump versions and changelog

Update the version in:

- `Cargo.toml` and `Cargo.lock` (or run `cargo build` to regenerate the lock)
- `npm/package.json` (root `version` **and** each `optionalDependencies` entry)
- `npm/packages/*/package.json` (all 6 subpackages)

npm versions carry a `-N` build suffix (e.g. `3.1.0-0`). Bump just the suffix
(`3.1.0-1`, ...) to republish the npm installers without a full version bump.

In `CHANGELOG.md`: rename `## Unreleased` to `## [3.1.0] - (YYYY-MM-DD)`, add a
fresh empty `## Unreleased`, and add the `[3.1.0]` / `[diff-3.1.0]` links
(point `[diff-unreleased]` at `v3.1.0...master`).

## 2. Verify and merge

- `cargo test --release`
- `cargo fmt --all -- --check` and `cargo clippy`
- Merge into `master` (changelog diff links reference it).

## 3. Tag and push

```sh
git tag -a v3.1.0 -m "v3.1.0: Support for elm 0.19.2"  # -s to sign
git push --follow-tags
```

## 4. GitHub release

Pushing a `v*` tag triggers the CI, which builds the binaries and creates a
**draft** release with executables + `checksums.txt` attached. Just edit the
notes and publish.

## 5. NPM release (manual)

Download the `npm-package` CI artifact and extract it. Check `npm whoami`
(should be `mattpiz`). Publish subpackages first, root last:

```sh
cd npm
# --access=public: @mattpiz scoped packages are private by default
# --tag latest: the -N suffix makes it a SemVer prerelease, so npm won't
#   auto-assign `latest`
for d in packages/*/; do (cd "$d" && npm publish --access=public --tag latest); done
npm publish --tag latest  # root, unscoped
```

npm can't republish the same version — if broken, bump the suffix (`3.1.0-1`).
Re-running the loop is safe (already-published subpackages error harmlessly).

## Crates.io

Maybe one day, with cargo-binstall?

## Community

Talk about the new features online.
