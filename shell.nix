/*
based on
https://discourse.nixos.org/t/how-can-i-set-up-my-rust-programming-environment/4501/9
*/
let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
pkgs.mkShell {
  buildInputs = [
    rust
  ] ++ (with pkgs; [
    pkg-config
    openssl
    # Analysis tools
    valgrind
    cargo-flamegraph
    # Node.js
    nodejs_20
    pnpm
    # for hot reloading in Cargo
    cargo-watch
  ]);
  packages = [
    (pkgs.python3.withPackages(python-pkgs: [
        python-pkgs.matplotlib
        python-pkgs.numpy
        python-pkgs.scikit-learn
    ]))
  ];
  RUST_BACKTRACE = 1;
}
