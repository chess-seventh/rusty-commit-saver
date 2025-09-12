{
  description =
    "A post-commit hook that saves commits to a specific file & directory";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustVersion = pkgs.rust-bin.stable."1.89.0".default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
      in {
        packages.default = rustPlatform.buildRustPackage {
          pname = "rusty-commit-saver";
          version = "1.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs;
            [
              pkg-config # Essential for finding OpenSSL
            ];

          buildInputs = with pkgs; [
            openssl # OpenSSL library
            openssl.dev # OpenSSL development headers
          ];

          # Environment variables to help find OpenSSL
          OPENSSL_NO_VENDOR = 1; # Use system OpenSSL instead of vendored
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rustVersion.override { extensions = [ "rust-src" ]; })
            cargo
            rustc
            pkg-config
            openssl
            openssl.dev
          ];

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_NO_VENDOR = 1;
        };
      });
}
