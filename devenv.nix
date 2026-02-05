{ pkgs, lib, config, inputs, ... }:

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

    cargo-edit # cargo add, cargo rm, cargo upgrade
    cargo-expand # cargo expand for macro debugging
    cargo-outdated # check for outdated dependencies
    cargo-audit # security audit
    cargo-deny # dependency management
    cargo-release # release management
    cargo-cross # cross-compilation
    cargo-machete # find unused dependencies
    cargo-update # update installed binaries
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
      after = [ "commitizen" "gitlint" "gptcommit" ];
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
      excludes = [ "^.envrc$" "^.direnv/.*" ];
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
        pkgs.nixfmt-classic
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
      settings.allFeatures = true;
      extraPackages = [ pkgs.openssl ];
      stages = [ "pre-commit" ];
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
        ${lib.generators.toKeyValue { }
        (lib.mapAttrs (name: value: value.description) config.scripts)}
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
      description = "Run tests with coverage";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "📊 Running tests with coverage..."
        cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
        cargo llvm-cov report
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

        # Remaining commits become the body (if any)
        if [ "$COMMIT_COUNT" -gt 1 ]; then
          PR_BODY=$(cat <<EOF
        ## Commits

        $(git log master..HEAD --reverse --pretty=format:'- %s' | tail -n +2)
        EOF
        )
        else
          # Single commit - use full commit message as body
          PR_BODY=$(git log -1 --pretty=format:'%b')

          # If body is empty, add a placeholder
          if [ -z "$PR_BODY" ]; then
            PR_BODY="<!-- Add PR description here -->"
          fi
        fi

        echo "📝 Creating PR with title:"
        echo "   $PR_TITLE"
        echo ""
        echo "📋 Body:"
        echo "$PR_BODY"
        echo ""
        echo "🚀 Pushing branch: $CURRENT_BRANCH"

        # Push current branch
        git push -u origin "$CURRENT_BRANCH"

        # Create PR
        gh pr create \
          --title "$PR_TITLE" \
          --body "$PR_BODY" \
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
    ${lib.generators.toKeyValue { }
    (lib.mapAttrs (name: value: value.description) config.scripts)}
    EOF
    echo
  '';

  enterTest = ''
    cargo clippy --all-targets -- -D warnings
    cargo llvm-cov --html nextest --no-fail-fast
    cargo nextest run --no-fail-fast --all-targets
  '';
}
