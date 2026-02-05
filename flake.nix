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

        # Use nightly to match your devenv.nix
        rustVersion = pkgs.rust-bin.nightly.latest.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
      in {
        packages.default = rustPlatform.buildRustPackage {
          pname = "rusty-commit-saver";
          version = "4.11.1";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl openssl.dev ];

          OPENSSL_NO_VENDOR = 1;
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          # Doctests are now marked ignore, so doCheck works fine
          doCheck = true;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (pkgs.rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" ];
            })
            cargo
            rustc
            pkg-config
            openssl
            openssl.dev
            git
          ];

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_NO_VENDOR = 1;
        };
      });
}
