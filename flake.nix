{
  description = "A post-commit hook that saves commits to a specific file & directory";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Use nightly to match your devenv.nix
        rustVersion = pkgs.rust-bin.nightly.latest.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };
      in
      {
        # `nix fmt` runs treefmt against ./treefmt.toml - the same config the
        # pre-commit hook uses. The toolchain comes from this flake's single
        # nixpkgs (unstable) so nix fmt is internally consistent (the L23 fleet
        # mitigation: pin the fmt toolchain to one nixpkgs source).
        #
        # INVARIANT: runtimeInputs must supply a binary for EVERY treefmt.toml
        # formatter that matches a tracked file here. treefmt.toml sets
        # `allow-missing-formatter = true` (fleet canon, one file fits all), so a
        # missing binary is skipped silently - and the CI gate below would then
        # pass without ever checking those files. Add a file type, add its
        # formatter here.
        formatter = pkgs.writeShellApplication {
          name = "treefmt-fmt";
          runtimeInputs = [
            pkgs.treefmt
            pkgs.git
            pkgs.nixfmt
            pkgs.deadnix
            pkgs.rustfmt
            pkgs.toml-sort
            pkgs.yamlfmt
            pkgs.markdownlint-cli # *.md
            pkgs.shfmt # *.sh
          ];
          text = "exec treefmt \"$@\"";
        };

        packages.default = rustPlatform.buildRustPackage {
          pname = "rusty-commit-saver";
          version = "4.17.5";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [
            openssl
            openssl.dev
          ];

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
      }
    );
}
