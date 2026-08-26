{
  pkgs,
  lib,
  config,
  ...
}:
let
  # The shared devenv modules live in the governed `devenv_shared` repository.
  # persona-bootstrap clones it to ~/src/claude-src/repos/devenv_shared; an older
  # maintainer layout kept a copy at ~/devenv_shared. NEITHER path exists on a
  # fresh box or on a CI runner, and this file used to import ~/devenv_shared/*
  # unconditionally - so `devenv shell` failed at EVALUATION and every gate
  # command documented in README.md was unreachable, with no error saying why.
  #
  # So: take the first directory that is actually present, take none when there
  # is none, and declare everything the gate needs below regardless. The shared
  # modules stay an enrichment (extra scripts, the shared git hooks), never a
  # precondition for the environment evaluating.
  sharedDirs = builtins.filter builtins.pathExists [
    "${builtins.getEnv "HOME"}/src/claude-src/repos/devenv_shared"
    "${builtins.getEnv "HOME"}/devenv_shared"
  ];
  sharedModules =
    if sharedDirs == [ ] then
      [ ]
    else
      builtins.filter builtins.pathExists (
        map (f: "${builtins.head sharedDirs}/${f}") [
          "shared_pkgs.nix"
          "shared_githooks.nix"
          "rust_pkgs.nix"
        ]
      );
in
{
  dotenv.enable = true;
  difftastic.enable = true;

  imports = sharedModules;

  env.GREET = "Welcome to the Rusty CV Commit Saver";

  packages = with pkgs; [
    sqlite
    postgresql
    # Codecov CLI for local baseline comparison
    # codecov-cli-bin

    # Self-sufficiency: without these the environment evaluates on a box with no
    # devenv_shared checkout but every script below still fails on a missing
    # binary, which is the same unreachable gate one step later.
    #
    # Test/lint tooling used by `pre-check` and `enterTest`.
    cargo-nextest
    cargo-shear
    cargo-llvm-cov

    # treefmt + its formatters, kept as the SAME list the flake's `formatter`
    # output carries, so `pre-check` and the CI formatting gate check the same
    # files. treefmt.toml sets `allow-missing-formatter = true`, so a missing
    # binary is skipped in silence and both gates report clean without ever
    # formatting those files. Add a file type, add its formatter in BOTH.
    #
    # KNOWN GAP, inherited from the flake and not closed here: treefmt.toml's
    # `prettier` matches *.json and four .json files are tracked, but neither
    # list declares prettier - so no tracked JSON is formatted by either gate.
    treefmt
    nixfmt
    deadnix
    toml-sort
    yamlfmt
    markdownlint-cli # *.md
    shfmt # *.sh
  ];

  languages = {
    nix.enable = true;
    shell.enable = true;

    # Kept byte-identical to devenv_shared/rust_pkgs.nix: when that module IS
    # imported both definitions are equal, which the module system merges; a
    # divergent channel or component list here would make the shared box fail
    # with a conflicting-definition error instead.
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
  };

  claude.code = {
    enable = true;
    hooks = {
      # Protect sensitive files (PreToolUse hook)
      protect-secrets = {
        enable = true;
        name = "Protect sensitive files";
        hookType = "PreToolUse";
        matcher = "^(Edit|MultiEdit|Write)$";
        command = ''
          # Read the JSON input from stdin
          json=$(cat)
          file_path=$(echo "$json" | jq -r '.file_path // empty')

          if [[ "$file_path" =~ \.(env|secret)$ ]]; then
            echo "Error: Cannot edit sensitive files"
            exit 1
          fi
        '';
      };

      # Log notifications (Notification hook)
      log-notifications = {
        enable = true;
        name = "Log Claude notifications";
        hookType = "Notification";
        command = ''mkdir -p .claude && echo "Claude notification received" >> .claude/claude.log'';
      };

      # Track completion (Stop hook)
      track-completion = {
        enable = true;
        name = "Track when Claude finishes";
        hookType = "Stop";
        command = ''mkdir -p .claude && echo "Claude finished at $(date)" >> .claude/claude-sessions.log'';
      };

      # Subagent monitoring (SubagentStop hook)
      subagent-complete = {
        enable = true;
        name = "Log subagent completion";
        hookType = "SubagentStop";
        command = ''mkdir -p .claude && echo "Subagent task completed" >> .claude/subagent.log'';
      };
    };
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
    ${lib.generators.toKeyValue { } (lib.mapAttrs (_name: value: value.description) config.scripts)}
    EOF
    echo
  '';

  enterTest = ''
    cargo clippy --all-targets -- -D warnings
    cargo llvm-cov --html nextest --no-fail-fast
    cargo nextest run --no-fail-fast --all-targets
  '';
}
