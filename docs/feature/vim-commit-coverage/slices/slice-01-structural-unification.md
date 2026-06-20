# Slice 01 — Structural unification (binary consumes the library) `@infrastructure`

> Elephant-carpaccio slice for feature `vim-commit-coverage`. Maps to **US-01**.
> Escape-valve note: this slice contains only `@infrastructure` stories. That is
> normally a hard-block (Dimension 0 slice-level), but this **entire feature is
> legitimately infrastructure** (Decision 1/4). The slice-composition gate is
> satisfied **via the escape valve**, not violated.

## Goal

Remove duplicate compilation of `vim_commit`/`config` so `cargo llvm-cov` measures
a single instrumented copy, then re-baseline the coverage number.

## IN scope

- Make `src/main.rs` a thin shim: `use rusty_commit_saver::{vim_commit::…, config::…}`.
- Drop the duplicate `pub mod vim_commit; pub mod config;` from `src/main.rs`.
- Re-run `devenv shell -- test-coverage` and record the new baseline.

## OUT of scope

- Any new test (that is Slices 02–04).
- Changing diary behavior, output, or config schema.
- Touching `config.rs` internals (already 99.66%).

## Learning hypothesis

We believe removing the dead duplicate copy will materially lift the measured
`vim_commit.rs` percentage **with zero new tests** — proven plausible because
`config.rs` already reaches 99.66% under the identical dual-crate setup. The
re-baseline tells us exactly how much genuine gap remains for Slices 02–04.

## Acceptance criteria

- [ ] `cargo llvm-cov` shows exactly one compilation each of `vim_commit.rs` / `config.rs`.
- [ ] No `vim_commit.rs` function-record reads zero solely due to duplicate compilation.
- [ ] `devenv shell -- cargo test` green in parallel; `devenv shell -- lint` clean; `devenv shell -- build-release` green.
- [ ] Diary output (frontmatter + table row) unchanged.

## Dependencies

None. This slice is the prerequisite for Slices 02–04.

## Effort

≤ 1 day.

## Reference class

Crate lib/bin unification refactors in small Rust projects — mechanical, low-risk,
well-understood; the main risk is relocating bin-side tests, addressed in 02–04.

## Carpaccio taste tests

- **End-to-end?** Yes — exercises the full measurement pipeline (`test-coverage` → number).
- **Demonstrable?** Yes — before/after coverage % and function-record count.
- **Thin?** Yes — structural change only, no new tests.
- **Independent value?** Yes — a trustworthy coverage number is valuable on its own.
