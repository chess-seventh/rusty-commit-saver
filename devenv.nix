{
  pkgs,
  lib,
  config,
  ...
}:
let
  # L235 - THE FLEET GATE, as four hooks of this repository's own (D-115).
  #
  # WHY A REPOSITORY WITH SIXTEEN HOOKS STILL NEEDS THESE. Two reasons, and the
  # second is the one that decided it:
  #
  #   1. `shared_githooks.nix` declares no `gitleaks`. This repository reaches
  #      the fleet's TODAY only because it sets no local `core.hooksPath`, so
  #      git resolves the fleet's GLOBAL one and runs the whole fleet gate here.
  #      L235 removes that value. Without these entries the repository keeps its
  #      sixteen hooks and quietly loses the one scanner that fires on a
  #      credential this machine has never seen.
  #   2. ⛔ THOSE SIXTEEN HOOKS ARE CONDITIONAL ON AN IMPURE PATH THAT DROPS IN
  #      SILENCE. `sharedModules` above is built from
  #      `builtins.getEnv "HOME"` filtered by `builtins.pathExists`: on a box
  #      where `~/src/claude-src/repos/devenv_shared` is not checked out - a
  #      fresh machine, a CI runner - the list is EMPTY and this repository
  #      declares no hooks at all, with no error. `rusty_cv_creator` records
  #      that this has already happened. These four entries are declared HERE,
  #      in this file, so the gate does not depend on that lookup succeeding.
  #
  # ⚠ AN EARLIER READ OF THIS FILE WAS WRONG AND THE CORRECTION IS WORTH
  # KEEPING. L235 first reported this repository as importing
  # `devenv_shared/git_hooks.nix`, which is commented out in full and evaluates
  # to `{}` - and concluded it was ungated. It imports `shared_githooks.nix`,
  # which carries the real block. `rusty_cv_creator` is the one that takes the
  # empty file. Two importers, two different lists, and only one of them is
  # real: that asymmetry is itself routed to Archon.
  #
  # ⚠ SEPARATE, AND NOT FIXED BY THIS FILE: the SHARED `.git/hooks` here holds
  # four prek shims whose `--config` points at a reaped worktree. They are
  # invisible while a global `core.hooksPath` means git never looks there; once
  # it does, prek exits 1 and every commit is refused. That is box-local state
  # no commit can reach - it comes out with `prek uninstall`, which rides L235's
  # activation as a non-optional step. This repository supplies the fleet's
  # commit diary, so it is the worst place for that to bite.
  #
  # Built through writeShellApplication so the script is shellchecked at build
  # time and the entries name a store path rather than a working-tree file.
  fleetGateHook = "${
    pkgs.writeShellApplication {
      name = "fleet-gate-hook";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.git
      ];
      text = builtins.readFile ./hooks/fleet-gate-hook;
    }
  }/bin/fleet-gate-hook";
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

  git-hooks.hooks = {
    # L235 - the fleet gate, reached as four of this repo's own hooks. The
    # rationale is in the `let` block at the top of this file; what matters here
    # is that these four are ordinary entries with nothing special about them.
    fleet-gate = {
      enable = true;
      name = "fleet gate";
      stages = [ "pre-commit" ];
      entry = "${fleetGateHook} pre-commit";
      language = "system";
      pass_filenames = false;
      always_run = true;
    };

    # pass_filenames, because git hands commit-msg the message file and the
    # fleet gate's gitlint and commitizen read it. Getting this wrong lints the
    # wrong thing while still exiting 0.
    fleet-gate-commit-msg = {
      enable = true;
      name = "fleet gate (message)";
      stages = [ "commit-msg" ];
      entry = "${fleetGateHook} commit-msg";
      language = "system";
      pass_filenames = true;
      always_run = true;
    };

    fleet-gate-pre-push = {
      enable = true;
      name = "fleet gate (push)";
      stages = [ "pre-push" ];
      entry = "${fleetGateHook} pre-push";
      language = "system";
      pass_filenames = false;
      always_run = true;
    };

    # The commit diary is hooks_everywhere.nix's post-commit hook - NOT
    # pkgs/git-commit-gate, which ships pre-commit and commit-msg only. On a box
    # with no fleet gate there is no diary, and the entry says so per commit.
    fleet-gate-post-commit = {
      enable = true;
      name = "fleet gate (diary)";
      stages = [ "post-commit" ];
      entry = "${fleetGateHook} post-commit";
      language = "system";
      pass_filenames = false;
      always_run = true;
    };

  };

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
