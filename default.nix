{ system ? builtins.currentSystem, pkgs ? import <nixpkgs> { system = system; }
, ... }:

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
#
# This configuration will automatically build for the current OS and arch.  If you
# want to compile for another system (by using a remote builder via nixbuild.net,
# for example), you can call this file like:
#
#     nix-build --arg system \"x86_64-linux\" .
#
# The quotes need to be escaped, since `--arg` takes a literal nix value without
# assuming it's a string.

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

  cargoSha256 = "00g8bnc9fbzmbia6cmgdg5g25g9yccwq2hldww8k2870r6dcz49m";
  verifyCargoDeps = true;
}
