# Evolution — vim-commit-coverage

**Shipped:** 2026-06-20 · **Waves:** DISCUSS → DESIGN → DISTILL → DELIVER (lean) · **Type:** infrastructure (escape valve)

## Outcome

`src/vim_commit.rs` line coverage **54% → 97.66%** (codecov 95% project gate now met). Full suite green in parallel (107 pass, 1 documented-ignored), clippy clean, release green. No change to user-facing diary behavior.

## What actually shipped

1. **The real fix (one line):** added `#[cfg_attr(coverage_nightly, coverage(off))]` to `vim_commit.rs`'s inline `mod commit_saver_tests`. This is the project's **own established convention** — `config.rs` (99.66%) and `main.rs` already annotate their test modules; `vim_commit.rs` was the lone module that omitted it, so its 456-line test module was wrongly counted in the coverage denominator. Excluding it dropped the denominator 653→128 real lines and revealed the true ~98% coverage.
2. **Lib/bin unification (ADR-001):** `main.rs` now consumes the library crate (`use rusty_commit_saver::{vim_commit, config}`) instead of re-declaring `pub mod`, removing dual compilation. Sound hexagonal cleanup, but a **minor** contributor to the number (lib-only was already ~44% before).
3. **`try_discover(path)` helper:** extracted from `try_new()` so the `Repository::discover` failure arm is testable without mutating process-global CWD. Activated `test_try_new_discovery_failure_blocked`.
4. **11 error-branch tests** authored in DISTILL (no-HEAD, detached HEAD, IO/permission failures, path inspection) — characterization tests of pre-existing behavior; all green on activation. One (`out_of_range_timestamp`) left `#[ignore]`d as documented-unreachable (libgit2 constrains stored commit times within chrono's range).

## Retrospective (lessons — this run was NOT clean)

- **Wrong root-cause hypothesis propagated through three waves.** DISCUSS/DESIGN/DISTILL all built on the initial theory that the coverage gap was dominated by lib/bin dual-compilation. DELIVER's first measurement (45%→52% after unification) falsified it. The *actual* cause — an unexcluded test module, diagnosable in one diff against `config.rs` — was only found by classifying the missed lines in DELIVER. **Lesson:** when a coverage number looks anomalous, diff the low-scoring file's coverage *config/annotations* against a high-scoring sibling in the same repo **before** designing a fix. A 5-minute comparison would have reframed the whole feature.
- **The "measurement vs. genuine gap" question should be settled empirically in DISCUSS**, not assumed. The escape-valve framing was right; the mechanism was guessed.
- **Crafter over-committed.** The step-01-01 crafter swept the entire working tree (logs, `.mcp.json`, pre-existing `devenv.nix`, DES state) into one commit. Resolved via soft-reset + `.gitignore` + clean re-commit. **Lesson:** under lean (no per-step commit needed), instruct crafters to stage explicit paths or not commit at all.
- **Tooling friction:** `nwave-ai outcomes` CLI is broken (missing `jsonschema`); `des-verify-integrity` roadmap pre-check has an `int()`-on-list bug but the core validator works once the `roadmap`/`phases` nesting is correct.

## Pointers

- Requirements + scenarios: `docs/feature/vim-commit-coverage/feature-delta.md`
- Architecture decision: `docs/product/architecture/adr-001-lib-bin-unification.md`
- DES audit: `docs/feature/vim-commit-coverage/deliver/{roadmap,execution-log}.json`
