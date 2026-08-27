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
- A **`git log` backstop** (`--reconcile`) that journals commits made on a
  machine with no vault mounted, so a row is late rather than lost — for the
  history `HEAD` reaches
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

### Building and testing

```bash
devenv shell -- cargo test               # the whole suite
devenv shell -- cargo clippy --all-targets
devenv shell -- pre-check                # linters + tests + build
```

> `devenv.nix` needs nothing outside the repository. It *enriches* the shell
> with the shared modules from the `devenv_shared` checkout when the machine
> has one — at `~/src/claude-src/repos/devenv_shared`, or at the older
> `~/devenv_shared` — and evaluates fine without it, which is what a fresh box
> and a CI runner get. Everything the commands above need (the Rust toolchain,
> `cargo-nextest`/`cargo-shear`/`cargo-llvm-cov`, `treefmt` and its formatters)
> is declared in `devenv.nix` itself, so this gate is reachable anywhere.
>
> The repository's own flake is self-contained too, so
> `nix develop --command cargo test` also works on any machine with Nix. Note
> that its `rustfmt` defaults to a different style edition — format with
> `cargo fmt -- --style-edition=2024` there, or it will reflow files it should
> leave alone.

#### The `devenv` input is pinned on purpose — do not unpin it

`devenv.yaml` pins the `devenv` module input to the release whose module version
equals the installed devenv CLI. Left unpinned, that input follows
`cachix/devenv`'s default branch, so `devenv update` locks modules **newer than
the CLI**, and `dotenv.enable = true` — which this repository sets — then fails
at evaluation with:

```text
The dotenv integration requires the C-Nix devenv CLI. It is not
available through the flake integration or another standalone Nix evaluation.
```

The failure lands on the **next** `direnv` load rather than on the update, so it
does not look like the update caused it. Recovery is `git restore devenv.lock`.

**The tag name is not the module version.** devenv compares `devenv version`
against the pinned module's `src/modules/latest-version`, and cachix's tags run
one release ahead of it — tag `v2.2.2` ships module `2.2.1`, tag `v2.2.1` ships
module `2.2.0`. So pin the tag whose `latest-version` **equals** the CLI, not
the tag of the same name:

| installed CLI | pin | module it locks |
| --- | --- | --- |
| `2.2.1` (the fleet today) | `ref=v2.2.2` | `2.2.1` |

Bump the pin only together with the fleet's devenv CLI, and **never above it**.
Equal is the only value that both keeps the environment and silences the notice.
A module *behind* the CLI still evaluates, but devenv keeps printing "run
`devenv update` to sync" — which is the prompt that caused this defect. A module
*ahead* of the CLI leaves the repository with no environment at all.

---

## Usage 🛞

Simply commit as usual. The hooks will:

1. Run linters (`clippy`, `rustfmt`, etc.) inside the Nix shell — **pre-commit**
2. Invoke Rusty Commit Saver to log the commit — **post-commit**, once the
   commit exists, which is why nothing this tool does can cost you a commit

If you prefer manual invocation:

```bash
rusty-commit-saver
```

Your commit will be appended to, where Obsidian should be:

```text
~/Documents/Wiki/📅 Diaries/0. Commits/YYYY/MM-MMMM/YYYY-MM-DD.md
```

### Catching up: the `git log` backstop

The hook can only journal on a machine that mounts the vault. Everywhere else
the row is lost, and once the config is present on those machines it is lost
*silently* — a journal that quietly stops looks exactly like a quiet week.

`--reconcile` is the other half: it reads the history each checkout's `HEAD`
can reach and appends whatever row the day note is missing. It needs no network
and no broker, so it works on the machine where the vault actually lives,
whenever you run it.

> **`HEAD`-reachable is narrower than "the repository", and the difference
> bites.** Commits sitting on a branch that is not the checkout's current
> `HEAD` are not journalled, and if that branch is later squash-merged they
> never become reachable at all. If you work in git worktrees, point
> `--reconcile` at each worktree rather than only at the main clone.

```bash
# everything each of these checkouts can reach from its own HEAD
rusty-commit-saver --reconcile ~/src/one --reconcile ~/src/two

# just the recent past, for a scheduled run
rusty-commit-saver --reconcile ~/src/one --since 2026-08-01
```

Each repository gets one line on stdout, so a scheduled run that appended
nothing still says so:

```text
rusty-commit-saver: one: 412 scanned, 3 appended, 409 already present
rusty-commit-saver: claude-src: excluded
```

What it will and will not do:

- **It only appends.** A row already in a note is never rewritten, reordered or
  reformatted. Your day notes are yours.
- **A row goes in the note for the commit's own date**, never today's — so a
  first backfill spreads across the months it actually happened in.
- **A row the hook already wrote is recognised**, by the commit hash in the
  last column, and not written twice. Running the pass a second time appends
  nothing and leaves every note byte-identical.
- **The `[exclude]` list applies here too.** An excluded repository is not even
  walked.
- **A repository it cannot open is reported on stderr and skipped**, so one
  broken clone does not stop the other repositories from catching up.
- **It ignores the directory you started it in.** Unlike the hook, it journals
  only the repositories you named.

`--since` takes a date as `YYYY-MM-DD`. A value it cannot read stops the run and
names both what it wanted and what it got, rather than quietly backfilling years
of history from a typo in a timer unit.

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

# Optional: repositories to skip, by canonical repo name (comma-separated).
# A commit made in one of these repos writes nothing to the diary.
[exclude]
repos = claude-src
```

The `[exclude]` section is optional. Each entry is matched, case-sensitively,
against the committing repository's **canonical name** — taken from its `origin`
remote URL (`…/claude-src.git` → `claude-src`), falling back to the
working-directory name for a repo with no usable `origin`. Because the origin is
the same from every checkout, one entry covers the main clone and every git
worktree of that repo, from any subdirectory.

`[obsidian]` and `[templates]` are required; a config missing either one is
fatal. Any **other** section is ignored, with a line on stderr naming it, never
fatal. One INI file is shared by every checkout on the machine, so a section
written for a newer release must not break a binary that predates it — which is
exactly what adding `[exclude]` did to every checkout older than 4.17.0.

The stderr line matters: a misspelt section (`[excludes]`) is ignored too, so
without it your exclusions would silently stop applying.

Keys work the same way, for the same reason:

- A key this binary does not understand is **ignored and named on stderr**
  (`ignoring unrecognised config keys [templates] commit_datetimes`). It used to
  be ignored in complete silence, so a typo applied nothing and said nothing.
- The four keys in `[obsidian]` and `[templates]` are **required**, and so is a
  non-empty value for each — `commit_path =` counts as missing. Without them
  there is no destination to write to, and a hook that quietly journals nothing
  looks exactly like a quiet day, so this one stays fatal. (`[exclude] repos` is
  optional, like its section.)
- The two `[templates]` values must be formats `chrono` can actually render,
  and that is checked when the config is read. A bad specifier used to surface
  from inside the writer as `a formatting trait implementation returned an
  error`, naming nothing, after an empty diary file had already been created.
- The fatal message names the config file, the key and its section, plus any
  unrecognised key in that same section, since a misspelt `commit_paths` is the
  usual reason `commit_path` is missing:

  ```text
  rusty-commit-saver: /home/you/.config/rusty-commit-saver/rusty-commit-saver.ini:
  missing required key 'commit_path' in section [obsidian];
  unrecognised in [obsidian]: commit_paths
  ```

None of this can cost you a commit: the tool runs as a **post-commit** hook, and
git ignores that hook's exit status. A config fault costs you the diary entry
and prints on stderr; the commit itself always stands.

### Checking hook behaviour by hand

`tests/hook-gate.sh` drives a real commit through a real post-commit hook, in a
throwaway repo and vault, and prints what a human would see:

```bash
cargo build
./tests/hook-gate.sh good             # journals, says nothing
./tests/hook-gate.sh unknown-key      # journals, names the key on stderr
./tests/hook-gate.sh missing-key      # journals nothing, names file + key
./tests/hook-gate.sh blank-key        # same, for a key with an empty value
./tests/hook-gate.sh bad-format       # same, for a format chrono cannot render
./tests/hook-gate.sh unknown-section  # journals, names the section
```

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
