# Feature brief — vim-commit-coverage

> Seed context for the nWave waves. Hand-authored from a grounded coverage
> analysis (2026-06-20). Waves may supersede this with their own artifacts under
> `discuss/`, `design/`, `distill/`, `deliver/`.

## Goal

Raise test coverage of `src/vim_commit.rs` to **≥ 95%** (the existing
`.codecov.yml` project target), aligning the currently-failing coverage gate
with reality.

## Current state (measured)

Overall project coverage **57.82%**. The gap is **entirely `src/vim_commit.rs`**:

| File | Coverage | Notes |
|------|----------|-------|
| `src/config.rs` | 99.66% | Reference quality — leave as-is |
| `src/main.rs` | 90.70% | **Ignored by codecov** (`.codecov.yml` `ignore:`) |
| `src/vim_commit.rs` | ~44% (lib) / 45.61% (combined) | **Target of this feature** |

Tooling: `cargo llvm-cov` via nextest. Run `devenv shell -- test-coverage`
(matches CI) or `devenv shell -- coverage-check`.

## Two tracks (both in scope)

### Track 1 — Structural / measurement
`src/main.rs` re-declares `pub mod vim_commit;` and `pub mod config;` instead of
consuming the library crate (`src/lib.rs` already exposes them via
`use rusty_commit_saver::…`). Every module therefore compiles **twice** (lib +
bin), and llvm-cov averages the tested copy against a dead, never-executed copy.
Verified: every production fn in `vim_commit.rs` executes (max exec count 9–14),
yet 33/63 function-records report zero — pure duplicate-compilation noise.

**Change:** make the binary a thin shim over the library — `main.rs` should
`use rusty_commit_saver::{vim_commit::…, config::…}` and drop the duplicate
`pub mod` declarations, so there is a single instrumented copy that the tests
cover. `config.rs` reaching 99.66% under the same dual-crate setup proves the
gap is closable.

### Track 2 — Genuine error-branch gaps
Measuring the library alone is still 43.66%, so real uncovered code remains —
concentrated in `# Errors` paths that need **fault injection**:

- `CommitSaver::from_repo` / `try_new` / `default` — no `HEAD`, detached `HEAD`
  (no branch shorthand), timestamp out of representable range.
- `append_entry_to_diary` — write/IO failure, parent missing.
- `create_diary_file` — file create/write failure.
- `create_directories_for_new_entry` — permission denied / read-only FS.
- `check_diary_path_exists` — path does not exist.
- `get_parent_from_full_path` — root path / no parent.

## Acceptance criteria

- `src/vim_commit.rs` line coverage ≥ 95% under `devenv shell -- test-coverage`.
- No duplicate compilation of `vim_commit`/`config` between bin and lib.
- All `# Errors` branches above have at least one covering test.
- Full suite green **in parallel** (`devenv shell -- cargo test`); clippy clean
  (`devenv shell -- lint`); release build green.

## Constraints / conventions

- Rigor profile: **lean** (`.nwave/des-config.json`).
- Build/test **only via devenv** — the bare shell has a mismatched clippy
  (0.1.86 vs rustc 1.94); devenv pairs them at 1.97. See `CLAUDE.md`.
- **No `std::env::set_current_dir` in tests** (process-global, races parallel
  tests). Inject via `CommitSaver::from_repo(&repo)` + `tempfile::tempdir()`.

## Proposed wave sequence

1. `/nw-discuss vim-commit-coverage` — bootstrap SSOT (greenfield), frame the
   goal + acceptance criteria above.
2. `/nw-design vim-commit-coverage` — decide & document the lib/bin unification
   (Track 1): binary as thin shim over the library crate.
3. `/nw-distill vim-commit-coverage` — author error-path acceptance tests
   (Track 2) with fault injection + PBT where it fits.
4. `/nw-deliver vim-commit-coverage` — implement the structural change + GREEN
   the tests via TDD, then verify the ≥95% coverage gate.
