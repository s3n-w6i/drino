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
        };

        vendorHash = "sha256-WMpq9EP4ksxkgz1/R25piVgTB5TZe1DdzyGful7sixI=";
    };
    # arro3_compute-0.4.5-pp310-pypy310_pp73-manylinux_2_17_x86_64.manylinux2014_x86_64.whl
    # arro3_compute-0.4.5-pp310-pypy310_pp73-manylinux_2_17_x86_64-manylinux2014_x86_64.whl
    arro3-compute = pkgs.python3Packages.buildPythonPackage rec {
        pname = "arro3-compute";
        version = "0.4.5";
        src = pkgs.fetchPypi {
            inherit version;
            pname = "arro3_compute";
            hash = "sha256-EhZM53j7H9F5sY2phBoplOhHeXG2OhG0K7FH2ZIrcHQ=";
            format = "wheel";
            python = "pp310-pypy310_pp73";
            abi = "manylinux_2_17_x86_64";
            platform = "manylinux2014_x86_64";
            # python = "pp310";
            # abi = "pypy310_pp73";
            # platform = "musllinux_1_2_x86_64";
        };
        doCheck = false; # don't run tests
        format = "pyproject";
    };
    arro3-core = pkgs.python3Packages.buildPythonPackage rec {
        pname = "arro3-core";
        version = "0.4.5";
        src = pkgs.fetchPypi {
            inherit version;
            pname = "arro3_core";
            hash = "sha256-EhZM53j7H9F5sY2phBoplOhHeXG2OhG0K7FH2ZIrcHQ=";
            # format = "wheel";
            # abi = "cp311-pp310-pypy310_pp73-manylinux_2_17_s390x";
            # platform = "manylinux2014_x86_64";
        };
        doCheck = false; # don't run tests
        format = "pyproject";
    };
    arro3-io = pkgs.python3Packages.buildPythonPackage rec {
        pname = "arro3-io";
        version = "0.4.5";
        src = pkgs.fetchPypi {
            inherit version;
            pname = "arro3_io";
            hash = "sha256-EhZM53j7H9F5sY2phBoplOhHeXG2OhG0K7FH2ZIrcHQ=";
            # format = "wheel";
            # abi = "cp311-pp310-pypy310_pp73-manylinux_2_17_s390x";
            # platform = "manylinux2014_x86_64";
        };
        doCheck = false; # don't run tests
        format = "pyproject";
    };
    lonboard = pkgs.python3Packages.buildPythonPackage rec {
        pname = "lonboard";
        version = "0.10.4";
        src = pkgs.fetchPypi {
            inherit pname version;
            hash = "sha256-EhZM53j7H9F5sY2phBoplOhHeXG2OhG0K7FH2ZIrcHQ=";
        };
        doCheck = false; # don't run tests
        format = "pyproject";
        buildInputs = [
            pkgs.python3Packages.poetry-core
            pkgs.python3Packages.anywidget
            arro3-compute
            arro3-core
            arro3-io
            pkgs.python3Packages.numpy
            pkgs.python3Packages.pyproj
            pkgs.python3Packages.traitlets
        ];
    };
    sidecar = pkgs.python3Packages.buildPythonPackage rec {
        pname = "sidecar";
        version = "0.7.0";
        src = pkgs.fetchPypi {
            inherit pname version;
            hash = "sha256-w/oWlLVhHB+rnXqWwHV3ugdkBoqJt7Ob6bnOl3Dp02M=";
        };
        doCheck = false; # don't run tests
        format = "pyproject";
        buildInputs = [
            pkgs.python3Packages.hatchling
            pkgs.python3Packages.hatch-jupyter-builder
            pkgs.python3Packages.hatch-nodejs-version
            pkgs.python3Packages.jupyterlab
            pkgs.python3Packages.ipywidgets
        ];
    };
in
(pkgs.mkShell.override {
    # use the faster mold linker
    stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.llvmPackages_19.stdenv;
}) {
    buildInputs = [
        rust
    ] ++ (with pkgs; [
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
    nativeBuildInputs = with pkgs; [
        openssl
        pkg-config
    ];
    packages = [
        (pkgs.python3.withPackages(python-pkgs: [
            python-pkgs.matplotlib
            python-pkgs.numpy
            python-pkgs.scikit-learn
            python-pkgs.pyarrow
            python-pkgs.jupyterlab
            python-pkgs.geopandas
            python-pkgs.pandas
            python-pkgs.shapely
            python-pkgs.palettable
            python-pkgs.ipywidgets
            # lonboard
            sidecar
        ]))
        gtfstidy
        pkgs.jupyter
    ];
    RUST_BACKTRACE = 1;
    # Compile for native CPU
    RUSTFLAGS = "-C target-cpu=native";
}
