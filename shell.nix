let
  # moz_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [  ]; };
  # ruststable = (nixpkgs.latest.rustChannels.stable.rust.override {
    # extensions = [ "rust-src" "rls-preview" "rust-analysis" "rustfmt-preview" ];
  # });
  pkgs = nixpkgs;  # Assign nixpkgs to pkgs for easier reference
in
  pkgs.mkShell {
    buildInputs = [
      pkgs.openssl
      pkgs.rustup
      # pkgs.rust-analyzer
      # ruststable
      pkgs.cmake
      pkgs.zlib
      pkgs.sqlite
    ];

    shellHook = ''
      export OPENSSL_DIR="${pkgs.openssl.dev}"
      export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
    '';
  }

