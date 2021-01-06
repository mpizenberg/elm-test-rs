{ pkgs ? import <nixpkgs> { }, ... }:

let macosDeps = [ pkgs.darwin.apple_sdk.frameworks.CoreServices ];
in pkgs.rustPlatform.buildRustPackage {
  pname = "elm-test-rs";
  version = "0.5.1";

  buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin macosDeps;

  # a nice addition here might be https://github.com/hercules-ci/gitignore.nix to
  # ignore files from git, which would prevent unnecessary rebuilds. But since
  # this is just for packaging for now, no need to bother with managing the
  # dependency!
  src = ./.;

  # to update this, set the string to all 0's, rebuild, and grab the new hash
  # that nix gives you
  cargoSha256 = "1p3fyzs5bkvyvzm5ns3azjb82m5dsafy2c481rxkm00vanadk1mi";
  verifyCargoDeps = true;
}
