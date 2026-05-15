{
  pkgs,
  imports,
  lib,
  config,
  inputs,
  ...
}:
let
  # Fetch and patch Codecov CLI binary for NixOS
  codecov-cli-bin = pkgs.stdenv.mkDerivation rec {
    pname = "codecov-cli";
    version = "0.7.5";

    src = pkgs.fetchurl {
      url = "https://cli.codecov.io/latest/linux/codecov";
      sha256 = "sha256-iTDEuzAlSkLz2MNAcGsb40CIXiDA31Fgok76LgMOZis=";
    };

    dontUnpack = true;

    nativeBuildInputs = [ pkgs.autoPatchelfHook ];

    # Runtime dependencies the binary needs
    buildInputs = [
      pkgs.stdenv.cc.cc.lib # libstdc++
      pkgs.zlib
      pkgs.glibc
    ];

    installPhase = ''
      runHook preInstall
      mkdir -p $out/bin
      cp $src $out/bin/codecov
      chmod +x $out/bin/codecov
      runHook postInstall
    '';

    meta = with lib; {
      description = "Codecov CLI";
      homepage = "https://cli.codecov.io";
      license = licenses.asl20;
      platforms = platforms.linux;
    };
  };

in
{
  dotenv.enable = true;
  difftastic.enable = true;

  imports = [
    "${builtins.getEnv "HOME"}/devenv_shared/shared_pkgs.nix"
    "${builtins.getEnv "HOME"}/devenv_shared/shared_githooks.nix"
    "${builtins.getEnv "HOME"}/devenv_shared/rust_pkgs.nix"
  ];

  env.GREET = "Welcome to the Rusty CV Commit Saver";

  packages = with pkgs; [
    zlib
    sqlite
    texlive.combined.scheme-small
    diesel-cli
    postgresql

    # Codecov CLI for local baseline comparison
    # codecov-cli-bin
  ];

  languages = {
    nix.enable = true;
    shell.enable = true;
  };

  scripts = {
    pre-check = {
      description = ''
        runs linters, tests, and builds to prepare commit/push (more extensively than pre-commit hook)
      '';
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail

        if [ -f .env.testing ]; then
            source .env.testing
        fi

        treefmt
        cargo clippy --all-targets -- -D warnings
        cargo shear --fix
        cargo llvm-cov --html nextest --no-fail-fast
      '';
    };

    codecov-compare = {
      description = "Compare local coverage against Codecov baseline";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail

        echo "🔍 Comparing local coverage with Codecov baseline..."
        echo ""

        # Check if lcov.info exists
        if [ ! -f "lcov.info" ]; then
          echo "⚠️  No lcov.info found. Running coverage first..."
          test-coverage
        fi

        # Show local coverage percentage
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        awk '
          /^LF:/ { lf += $2 }
          /^LH:/ { lh += $2 }
          END {
            if (lf > 0) {
              pct = lh * 100 / lf
              printf "📊 Local Coverage: %.2f%% (%d / %d lines)\n", pct, lh, lf
            }
          }
        ' FS=: lcov.info
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""

        # Check for CODECOV_TOKEN
        if [ -z "''${CODECOV_TOKEN:-}" ]; then
          echo "ℹ️  CODECOV_TOKEN not set — skipping Codecov upload validation"
          echo "   Set it in .env or export it to enable upload testing"
          exit 0
        fi

        echo "📤 Validating upload (dry-run)..."
        codecov do-upload \
          --token="$CODECOV_TOKEN" \
          --slug=chess-seventh/rusty-commit-saver \
          --file=lcov.info \
          --flag=local \
          --dry-run 2>&1 | grep -E "(Found|coverage|complete|error)" || true

        echo ""
        echo "✅ Local coverage ready. Compare with: https://app.codecov.io/gh/chess-seventh/rusty-commit-saver"
      '';
    };

    coverage-check = {
      description = "Show coverage percentage (quick check)";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail

        if [ ! -f "lcov.info" ]; then
          echo "⚠️  No lcov.info found. Run 'test-coverage' first."
          exit 1
        fi

        echo "📊 Local Coverage Summary:"
        echo ""
        cargo llvm-cov report 2>/dev/null || {
          # Fallback: parse lcov.info directly
          LINES_HIT=$(grep -c "^DA:" lcov.info || echo 0)
          LINES_FOUND=$(grep "^DA:" lcov.info | cut -d',' -f2 | grep -c "0" || echo 0)
          echo "Lines in lcov.info: $LINES_HIT"
        }
      '';
    };
  };

  enterShell = ''
    echo "Sourcing .env with evaluated command substitution…"
    if [ -f ".env" ]; then
      eval "$(<.env)"
    fi

    echo
    echo 💡 Helper scripts to ease development process:
    echo
    ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|• |' -e 's|••| |g'
    ${lib.generators.toKeyValue { } (lib.mapAttrs (name: value: value.description) config.scripts)}
    EOF
    echo
  '';

  enterTest = ''
    cargo clippy --all-targets -- -D warnings
    cargo llvm-cov --html nextest --no-fail-fast
    cargo nextest run --no-fail-fast --all-targets
  '';
}
