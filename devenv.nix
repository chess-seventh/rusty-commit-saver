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

  # processes = { cargo-watch.exec = "cargo-watch"; };

  # tasks = {
  #   "bash:source_env" = {
  #     exec = "source $PWD/.env";
  #     after = [ "devenv:enterShell" ];
  #   };
  # };

  git-hooks.hooks = {
    rusty-commit-saver = {
      enable = true;
      name = "🦀 Rusty Commit Saver";
      stages = [ "post-commit" ];
      after = [ "commitizen" "gitlint" "gptcommit" ];
      entry = "${
          inputs.rusty-commit-saver.packages.${pkgs.system}.default
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

  };

  scripts = {
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
        cargo clippy --all-targets -- -W clippy::pedantic -A clippy::missing_errors_doc -A clippy::must_use_candidate -A clippy::module_name_repetitions -A clippy::doc_markdown -A clippy::missing_panics_doc
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

    build = {
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

    test = {
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

    check = {
      description = "Check code without building";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "✅ Checking code..."
        cargo check
      '';
    };

    audit = {
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
