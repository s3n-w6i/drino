/*
based on
https://discourse.nixos.org/t/how-can-i-set-up-my-rust-programming-environment/4501/9
*/
let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rustVersion = "1.76.0";
  rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
pkgs.mkShell {
  buildInputs = [
    rust
  ] ++ (with pkgs; [
    pkg-config
    valgrind
    cargo-flamegraph
  ]);
  RUST_BACKTRACE = 1;
}