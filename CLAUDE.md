# rusty-commit-saver — project guide

A small Rust CLI, run as a Git hook, that records each commit's metadata
(folder, time, message, repo URL, branch, hash) as a Markdown table row in a
dated Obsidian diary entry. Core logic lives in `src/vim_commit.rs`
(`CommitSaver`); the CLI entry point and `run_commit_saver` are in `src/main.rs`.

## Build / test — always via devenv

The repo's toolchain is pinned by `devenv.nix`. **Run cargo through the devenv
shell**, not the bare shell:

```sh
devenv shell -- cargo test          # or the script: devenv shell -- test-cargo
devenv shell -- cargo clippy --all-targets   # or: devenv shell -- lint
devenv shell -- cargo build --release        # or: devenv shell -- build-release
devenv shell -- pre-check           # linters + tests + build, before commit/push
```

Useful devenv scripts (see `devenv shell -- devhelp`): `test-cargo`, `lint` /
`cclippy`, `build-project`, `build-release`, `check-code`, `test-coverage`
(matches CI), `pre-check`.

> Gotcha: the bare login shell has a **mismatched Rust toolchain** — `rustc`
> 1.94 but `clippy-driver` 0.1.86 — so `cargo clippy` outside devenv fails with
> `E0514: found crate ... compiled by an incompatible version of rustc`. Inside
> the devenv shell rustc and clippy are matched (1.97). `cargo test` / `cargo
> build` happen to work in the bare shell, but clippy does not. Use devenv.

## Testing conventions

- **Never mutate the process-global current directory in tests**
  (`std::env::set_current_dir`). Tests run in parallel and the CWD is shared
  process state — mutating it races with any test that calls
  `Repository::discover("./")` and produces intermittent
  `reference 'HEAD' not found` panics.
- To exercise `CommitSaver` against a specific repo, build a `git2::Repository`
  in a `tempfile::tempdir()` and call **`CommitSaver::from_repo(&repo)`** —
  the injectable, no-ambient-state constructor. Reserve `CommitSaver::new()` /
  `default()` (which discover `./`) for the binary's real run path.
- `CommitSaver::try_new()` is the non-panicking variant of `new()`/`default()`.

## Development Paradigm

This project follows the **object-oriented / imperative** paradigm (struct-based:
`CommitSaver` aggregate with methods; straightforward function signatures; no
trait-heavy abstraction). Use `@nw-software-crafter` for implementation.

## nWave

This project is nWave-managed (lightweight adoption). Rigor profile: **lean**
(`.nwave/des-config.json`, local-only — the whole `.nwave/` dir is gitignored as
generated state). Change it with `/nw-rigor`. Wave artifacts are written under
`docs/feature/<feature>/` and archived to `docs/evolution/` on finalize. Ask
`/nw-buddy` anything about the methodology or project state.
