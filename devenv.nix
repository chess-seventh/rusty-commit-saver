{
  pkgs,
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
      sha256 = "0v1zmw25f6z8xcn1zrdfx787bsf47v1b545cwd48wwrc5d722d7x";
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

  env.GREET = "Welcome to the Rusty CV Commit Saver";

  starship = {
    enable = true;
    config = {
      enable = false;
      path = "~/.config/starship.toml";
    };
  };

  packages = with pkgs; [
    git
    jq
    curl
    gnused
    zlib
    sqlite
    texlive.combined.scheme-small
    diesel-cli
    postgresql
    cargo-nextest
    cargo-shear
    cargo-llvm-cov
    cargo-watch
    rustup
    bacon
    gh

    cargo-deny
    cargo-edit # cargo add, cargo rm, cargo upgrade
    cargo-expand # cargo expand for macro debugging
    cargo-outdated # check for outdated dependencies
    cargo-audit # security audit
    cargo-deny # dependency management
    cargo-release # release management
    cargo-cross # cross-compilation
    cargo-machete # find unused dependencies
    cargo-update # update installed binaries

    # Codecov CLI for local baseline comparison
    codecov-cli-bin
  ];

  languages = {
    nix.enable = true;

    rust = {
      enable = true;
      channel = "nightly";
      components = [
        "rustc"
        "cargo"
        "clippy"
        "rustfmt"
        "rust-analyzer"
        "rust-std"
        "llvm-tools-preview"
      ];
    };

    shell.enable = true;
  };

  git-hooks.hooks = {
    rusty-commit-saver = {
      enable = true;
      name = "🦀 Rusty Commit Saver";
      stages = [ "post-commit" ];
      after = [
        "commitizen"
        "gitlint"
        "gptcommit"
      ];
      entry = "${
        inputs.rusty-commit-saver.packages.${pkgs.stdenv.hostPlatform.system}.default
      }/bin/rusty-commit-saver";
      pass_filenames = false;
      language = "system";
      always_run = true;
    };

    check-merge-conflicts = {
      name = "🔒 Check Merge Conflicts";
      enable = true;
      stages = [ "pre-commit" ];
    };

    detect-aws-credentials = {
      name = "💭 Detect AWS Credentials";
      enable = true;
      stages = [ "pre-commit" ];
    };

    detect-private-keys = {
      name = "🔑 Detect Private Keys";
      enable = true;
      stages = [ "pre-commit" ];
    };

    end-of-file-fixer = {
      name = "🔚 End of File Fixer";
      enable = true;
      stages = [ "pre-commit" ];
    };

    mixed-line-endings = {
      name = "🔀 Mixed Line Endings";
      enable = true;
      stages = [ "pre-commit" ];
    };

    trim-trailing-whitespace = {
      name = "✨ Trim Trailing Whitespace";
      enable = true;
      stages = [ "pre-commit" ];
    };

    shellcheck = {
      name = "✨ Shell Check";
      enable = true;
      stages = [ "pre-commit" ];
      excludes = [
        "^.envrc$"
        "^.direnv/.*"
      ];
    };

    mdsh = {
      enable = true;
      name = "✨ MDSH";
      stages = [ "pre-commit" ];
    };

    treefmt = {
      name = "🌲 TreeFMT";
      enable = true;
      settings.formatters = [
        pkgs.nixfmt
        pkgs.deadnix
        pkgs.yamlfmt
        pkgs.rustfmt
        pkgs.toml-sort
      ];
      stages = [ "pre-commit" ];
    };

    clippy = {
      name = "✂️ Clippy";
      enable = true;
      entry = "cargo clippy --all-targets -- -W clippy::pedantic -A clippy::must-use-candidate";
      language = "system";
      settings.allFeatures = true;
      extraPackages = [ pkgs.openssl ];
      stages = [ "pre-commit" ];
      pass_filenames = false;
    };

    commitizen = {
      name = "✨ Commitizen";
      enable = true;
      stages = [ "post-commit" ];
    };

    gptcommit = {
      name = "🤖 GPT Commit";
      enable = true;
    };

    gitlint = {
      name = "✨ GitLint";
      enable = true;
      after = [ "gptcommit" ];
    };

    markdownlint = {
      name = "✨ MarkdownLint";
      enable = true;
      stages = [ "pre-commit" ];
      excludes = [ "CHANGELOG.md" ];
      settings.configuration = {
        MD033 = false;
        MD013 = {
          line_length = 120;
          tables = false;
        };
        MD041 = false;
      };
    };

    pre-commit-shear = {
      name = "✨ Cargo Dependency Check";
      enable = true;
      # this is a simple shell hook
      entry = ''
        echo "Running cargo-shear pre-commit check..."
        if ! command -v cargo-shear >/dev/null 2>&1; then
          echo "cargo-shear not installed. Run: cargo install cargo-shear --locked"
          exit 1
        fi

        # Only run if there are staged changes in Cargo.toml or Cargo.lock
        if git diff --cached --name-only | grep -Eq '^Cargo\.toml$|^Cargo\.lock$'; then
          cargo shear
        else
          echo "No dependency files changed, skipping cargo-shear."
        fi
      '';
      language = "system";
      stages = [ "pre-commit" ];
    };
  };

  scripts = {
    devhelp = {
      description = "Shows all available commands in Devenv";
      exec = ''
        echo
        echo 💡 Helper scripts to ease development process:
        echo
        ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|• |' -e 's|••| |g'
        ${lib.generators.toKeyValue { } (lib.mapAttrs (name: value: value.description) config.scripts)}
        EOF
      '';
    };

    install_pre_hooks = {
      description = "Install Pre Hooks, such as gptcommit";
      exec = ''
        #!/usr/bin/env bash
        set -euxo pipefail
        gptcommit install
        gptcommit config set openai.model gpt-4-turbo
        gptcommit config set output.conventional_commit true
      '';
    };

    cclippy = {
      description = ''
        Run clippy
      '';
      exec = ''
        cargo clippy --all-targets -- -W clippy::pedantic -A clippy::must-use-candidate
      '';
    };

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

    watch-clippy = {
      description = "Watch and re-run tests on file changes";
      exec = ''
        bacon clippy
      '';
    };
    # cargo watch -x 'clippy --all-targets -- -D warnings' -x 'llvm-cov --html nextest --no-fail-fast'

    watch-coverage = {
      description = "Watch and re-run nextest on file changes";
      exec = ''
        bacon coverage
      '';
    };
    # cargo watch -x 'nextest run --no-fail-fast --all-targets'

    watch-check = {
      description = "Watch and run quick checks (clippy only)";
      exec = ''
        cargo watch -x 'clippy --all-targets -- -D warnings'
      '';
    };

    build-project = {
      description = "Build the Rust project";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🔨 Building Rust project..."
        cargo build
      '';
    };

    build-release = {
      description = "Build the Rust project in release mode";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🚀 Building Rust project (release mode)..."
        cargo build --release
      '';
    };

    test-cargo = {
      description = "Run tests with cargo test";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🧪 Running tests..."
        cargo test
      '';
    };

    test-coverage = {
      description = "Run tests with coverage (matches CI exactly)";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "📊 Running tests with coverage (CI-equivalent)..."

        # Clean previous coverage data
        cargo llvm-cov clean --workspace

        # Run tests with nextest (same as CI)
        cargo llvm-cov --no-report nextest --no-fail-fast

        # Generate lcov report (same as CI)
        cargo llvm-cov report --lcov --output-path lcov.info

        # Also generate human-readable summary
        echo ""
        echo "📈 Coverage Summary:"
        cargo llvm-cov report

        echo ""
        echo "✅ Coverage report saved to: lcov.info"
      '';
    };

    test-coverage-html = {
      description = "Run tests with coverage and open HTML report";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "📊 Running tests with coverage (HTML)..."
        cargo llvm-cov clean --workspace
        cargo llvm-cov --no-report nextest --no-fail-fast
        cargo llvm-cov report --html
        cargo llvm-cov report
        echo ""
        echo "📂 HTML report: target/llvm-cov/html/index.html"
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

    lint = {
      description = "Run Clippy linter";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🔍 Running Clippy linter..."
        cargo clippy --all-targets --all-features -- -D warnings
      '';
    };

    format = {
      description = "Format code with rustfmt";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🎨 Formatting code..."
        cargo fmt
      '';
    };

    check-code = {
      description = "Check code without building";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "✅ Checking code..."
        cargo check
      '';
    };

    audit-cargo = {
      description = "Security audit with cargo audit";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🔒 Running security audit..."
        cargo audit
      '';
    };

    outdated = {
      description = "Check for outdated dependencies";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "📦 Checking for outdated dependencies..."
        cargo outdated
      '';
    };

    clean = {
      description = "Clean build artifacts";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🧹 Cleaning build artifacts..."
        cargo clean
      '';
    };

    deps = {
      description = "Show dependency tree";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "📦 Dependency tree:"
        cargo tree
      '';
    };

    create-pr = {
      description = "Create GH PR with first commit as title, rest as body";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail

        # Check we're not on master
        CURRENT_BRANCH=$(git branch --show-current)
        if [ "$CURRENT_BRANCH" = "master" ]; then
          echo "❌ Cannot create PR from master branch"
          exit 1
        fi

        # Get all commits not in master
        COMMIT_COUNT=$(git rev-list --count master..HEAD)

        if [ "$COMMIT_COUNT" -eq 0 ]; then
          echo "❌ No commits to create PR from"
          exit 1
        fi

        # First commit (oldest) becomes the title
        PR_TITLE=$(git log master..HEAD --reverse --pretty=format:'%s' | head -1)

        echo "📝 Creating PR with title:"
        echo "   $PR_TITLE"
        echo ""
        echo "🚀 Pushing branch: $CURRENT_BRANCH"

        # Push current branch
        git push -u origin "$CURRENT_BRANCH"

        # Create PR
        gh pr create \
          --title "$PR_TITLE" \
          --body "" \
          --base master

        echo ""
        echo "✅ PR created successfully!"
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
