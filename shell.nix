/*
based on
https://discourse.nixos.org/t/how-can-i-set-up-my-rust-programming-environment/4501/9
*/
let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  gtfstidy = pkgs.buildGoModule rec {
    pname = "gtfstidy";
    version = "0.2";

    src = pkgs.fetchFromGitHub {
      owner = "patrickbr";
      repo = "gtfstidy";
      rev = "92a23c8dd16bdd2691468090c554f036c19df590";
      sha256 = "sha256-xNO3KEMPLJ3ygeXYIs7J9AWq9xq9OMUKS0kUI72iIxE=";
      # sha256 = "";
    };

    vendorHash = "sha256-WMpq9EP4ksxkgz1/R25piVgTB5TZe1DdzyGful7sixI=";
  };
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
    hyperfine
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
        python-pkgs.pyarrow
    ]))
    gtfstidy
  ];
  RUST_BACKTRACE = 1;
}
