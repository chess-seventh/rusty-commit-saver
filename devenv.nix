{ pkgs, lib, config, inputs, ... }:

{
  dotenv.enable = true;

  env.GREET = "Welcome to the Rusty CV Commit Saver";
  # env.OPENSSL_DIR="${pkgs.openssl.dev}";
  # env.OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib";

  starship = {
    enable = true;
    config = {
      enable = false;
      path = "~/.config/starship.toml";
    };
  };

  # https://devenv.sh/packages/
  packages = with pkgs; [
    git
    jq
    curl
    gnused
    # pkgs.rustup
    # pkgs.rust-analyzer
    # ruststable
    zlib
    sqlite
    texlive.combined.scheme-small
    diesel-cli
    postgresql
    cargo-nextest
    cargo-shear
    cargo-llvm-cov
    # cmake
    # gcc
    # openssl
  ];

  # https://devenv.sh/languages/
  languages = {
    nix.enable = true;

    rust = {
      enable = true;
      channel = "stable";
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

  # https://devenv.sh/processes/
  processes = { cargo-watch.exec = "cargo-watch"; };

  tasks = {
    "bash:source_env" = {
      exec = "source $PWD/.env";
      after = [ "devenv:enterShell" ];
    };
  };

  git-hooks.hooks = {
    check-merge-conflicts.enable = true;

    detect-aws-credentials.enable = true;

    detect-private-keys.enable = true;

    end-of-file-fixer.enable = true;

    mixed-line-endings.enable = true;

    no-commit-to-branch.enable = true;

    # typos.enable = true;

    treefmt = {
      enable = true;
      settings.formatters = [ pkgs.nixfmt-classic pkgs.deadnix ];
    };

    trim-trailing-whitespace.enable = true;

    shellcheck.enable = true;

    mdsh.enable = true;

    rustfmt.enable = true;

    clippy = {
      enable = true;
      packageOverrides.cargo = pkgs.cargo;
      packageOverrides.clippy = pkgs.clippy;
      # some hooks provide settings
      settings.allFeatures = true;
      extraPackages = [ pkgs.openssl ];
    };

    commitizen.enable = true;

    gptcommit = { enable = true; };

    gitlint = {
      enable = true;
      after = [ "gptcommit" ];
    };

    markdownlint = {
      enable = true;
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

        cargo fmt --all --check
        cargo clippy --all-targets -- -D warnings
        cargo shear --fix
        cargo llvm-cov --html nextest --no-fail-fast 
      '';
    };
  };

  enterShell = ''
    echo
    echo 💡 Helper scripts to ease development process:
    echo
    ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|• |' -e 's|••| |g'
    ${lib.generators.toKeyValue { }
    (lib.mapAttrs (name: value: value.description) config.scripts)}
    EOF
    echo
  '';

  # See full reference at https://devenv.sh/reference/options/
}
