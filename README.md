# 🦀 Rusty Commit Saver

<div align="center">

[![Rust](https://img.shields.io/badge/🦀%20rust-blue)](https://rustlang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-rustdoc-blue)](https://chess-seventh.github.io/rusty-commit-saver/rusty_commit_saver/)
[![codecov](https://codecov.io/github/chess-seventh/rusty-commit-saver/graph/badge.svg?token=4ZK40EALQ8)](https://codecov.io/github/chess-seventh/rusty-commit-saver)
[![Codecov Test Analytics](https://img.shields.io/badge/Codecov-Test%20Analytics-brightgreen)](https://app.codecov.io/gh/chess-seventh/rusty-commit-saver/analytics/tests)

[![🔍 CI - Quality & Coverage](https://github.com/chess-seventh/rusty-commit-saver/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/chess-seventh/rusty-commit-saver/actions/workflows/ci.yml)
[![🎯 Release](https://github.com/chess-seventh/rusty-commit-saver/actions/workflows/release.yml/badge.svg?branch=master)](https://github.com/chess-seventh/rusty-commit-saver/actions/workflows/release.yml)
[![📚 Documentation](https://github.com/chess-seventh/rusty-commit-saver/actions/workflows/docs.yml/badge.svg?branch=master)](https://github.com/chess-seventh/rusty-commit-saver/actions/workflows/docs.yml)

---

</div>

✨ A Rust flake to automatically log al Git commits into Obsidian. ✨

Rusty Commit Saver captures each commit’s:

- **Timestamp**
- **Commit message**
- **Repository URL**
- **Branch name**
- **Commit hash**

and appends it to a dated diary entry in your Wiki directory.

**Table of Contents:**

- [🚀 Features](#features-)
- [📦 Installation](#installation-)
- [🛞 Usage](#usage-)
- [🛠️ Configuration](#configuration-)
- [📈 Roadmap & Improvements](#roadmap--improvements-)
- [💖 Contributing](#contributing-)

---

## Features 🚀

- Automatic diary entry creation with YAML frontmatter and table header
- Timestamped commit rows formatted for Obsidian
- Customizable storage path under `📅 Diaries/0. Commits/YYYY/MM-MMMM/`
- Preconfigured hooks (via Nix + pre-commit) to ensure code quality

---

## Installation 📦

1. Clone the repository

   ```bash
   git clone https://github.com/chess-seventh/rusty-commit-saver.git
   cd rusty-commit-saver
   ```

2. Enter the Nix development shell

   ```bash
   devenv shell
   ```

3. Build the project

   ```bash
   cargo build --release
   ```

4. (Optional) Install the binary to your PATH

   ```bash
   cargo install --path .
   ```

---

## Usage 🛞

Simply commit as usual. The pre-commit hook will:

1. Run linters (`clippy`, `rustfmt`, etc.) inside the Nix shell
2. Invoke Rusty Commit Saver to log the commit

If you prefer manual invocation:

```bash
rusty-commit-saver
```

Your commit will be appended to, where Obsidian should be:

```text
~/Documents/Wiki/📅 Diaries/0. Commits/YYYY/MM-MMMM/YYYY-MM-DD.md
```

---

## Configuration 🛠

- **`rust-toolchain.toml`** pins Rust 1.89.0
- **`devenv.nix`** provisions Rust, Clippy, rustfmt, and Git hooks
- **`.pre-commit-config.yaml`** defines all pre-commit checks
- **`treefmt.toml`** configures `treefmt` and formatters

### Runtime config (INI)

Runtime settings live in an INI file at
`~/.config/rusty-commit-saver/rusty-commit-saver.ini`:

```ini
[obsidian]
root_path_dir = ~/Documents/Obsidian
commit_path = Diaries/Commits

[templates]
commit_date_path = %Y/%m-%B/%F.md
commit_datetime = %H:%M:%S

# Optional: repositories to skip, by working-directory name (comma-separated).
# A commit made in one of these repos writes nothing to the diary.
[exclude]
repos = claude-src
```

The `[exclude]` section is optional. Each entry is matched, case-sensitively,
against the committing repository's working-directory name (e.g. `claude-src`
for a repo checked out at `~/src/claude-src`) — so it holds no matter which
subdirectory the commit is made from.

`[obsidian]` and `[templates]` are required; a config missing either one is
fatal. Any **other** section is ignored, with a line on stderr naming it, never
fatal. One INI file is shared by every checkout on the machine, so a section
written for a newer release must not break a binary that predates it — which is
exactly what adding `[exclude]` did to every checkout older than 4.17.0.

The stderr line matters: a misspelt section (`[excludes]`) is ignored too, so
without it your exclusions would silently stop applying.

---

## Roadmap & Improvements 📈

There are **many enhancements** planned:

- Configurable Obsidian path
- Configurable year/day/month on where to save the commit
- Interactive CLI flags and richer metadata (author, files changed)
- Improved error handling and user feedback
- Unit tests and CI pipeline for automated releases

Contributions welcome! Feel free to open issues or submit PRs.

---

## Contributing 💖

1. Fork the repo
2. Create a feature branch
3. Write tests and update `README.md`
4. Submit a pull request

---

## License 📄

MIT © 2026 [Chess7th](mailto:chess7th@pm.me)
