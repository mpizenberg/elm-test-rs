{ pkgs ? import <nixpkgs> { }, ... }:

# how to keep this file up to date:
#
# 1. keep the `version` in sync with the version in `cargo.toml`. Nothing bad
#    will happen if you don't do this, but it's nicer to get bug reports
# 2. rebuild with nix whenever you update `cargo.lock` to get the new
#    `cargoSha256`. It will give you a more useful error if you replace the hash
#    with all 0s before building.
#
# In the longer term, this file could be submitted to nixpkgs to make it easier
# for Nix users to install elm-test-rs. That's probably the right move once it
# hits 1.0.0!

let macosDeps = [ pkgs.darwin.apple_sdk.frameworks.CoreServices ];
in pkgs.rustPlatform.buildRustPackage {
  pname = "elm-test-rs";
  version = "0.6.1";

  # a nice addition here might be https://github.com/hercules-ci/gitignore.nix to
  # ignore files from git, which would prevent unnecessary rebuilds. But since
  # this is just for packaging for now, no need to bother with managing the
  # dependency!
  src = ./.;

  buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin macosDeps;

  cargoSha256 = "16bx4yw5qnjilfkysxnmylirlrfaw4dri8xxdbp67n1b019sf8hg";
  verifyCargoDeps = true;
}
