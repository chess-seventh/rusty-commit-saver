<!-- markdownlint-disable MD024 -->
# Feature Delta — vim-commit-coverage (Wave: DISCUSS)

> Single-source narrative for the DISCUSS wave. Density: **lean** (Tier-1 [REF]
> sections only). Feature type: **Infrastructure** (escape-valve branch — JTBD
> skipped). Source of truth for technical scope: `docs/feature/vim-commit-coverage/brief.md`.

## Wave: DISCUSS / [REF] Persona ID

**Sam — the maintainer / CI pipeline.** Sam keeps `rusty-commit-saver` green and
releasable. Sam's instruments are the devenv coverage scripts
(`devenv shell -- test-coverage`, `devenv shell -- coverage-check`) and the
codecov **project** gate (`target: 95%` in `.codecov.yml`). Sam currently sees a
**failing** gate (project coverage 57.82%) that does not reflect the true quality
of `src/vim_commit.rs`, and cannot tell real gaps from measurement noise.

## Wave: DISCUSS / [REF] JTBD One-Liner

**infrastructure-only.** No user-facing Job To Be Done applies. This is a
test-coverage correctness + measurement feature serving the maintainer/CI, not an
end-user of the diary tool. Every story below carries `job_id: infrastructure-only`
plus an `infrastructure_rationale`, per the escape valve (Decision 4).

## Wave: DISCUSS / [REF] Locked Decisions

| # | Decision | Verdict | Consequence |
|---|----------|---------|-------------|
| D1 | Feature type | **Infrastructure** | Stories labelled `@infrastructure`; escape valve in effect |
| D2 | Walking skeleton | **No** | Brownfield, isolated; app already runs end-to-end. WS strategy = N/A |
| D3 | UX research depth | **Lightweight** | One-line maintainer persona; no emotional arc, no TUI mockups |
| D4 | JTBD | **No (infrastructure-only)** | Phase 1 (JTBD / four-forces / opportunity scoring) skipped; `job_id: infrastructure-only` + rationale on every story |

## Wave: DISCUSS / [REF] Scope Assessment (Elephant Carpaccio)

PASS — 4 stories, 1 bounded context (`src/vim_commit.rs` measurement + tests),
estimated ~3 days total. Right-sized; no split required. Sliced into 4
elephant-carpaccio briefs (`slices/slice-01..04`).

Slice-composition note: a slice containing only `@infrastructure` stories is
normally a hard-block (Dimension 0, slice-level check). This **entire feature is
legitimately infrastructure** under the escape valve (D1/D4). The block is
**satisfied via the escape valve, not violated** — documented in each slice brief
and acknowledged here so the reviewer applies the exemption.

---

## Wave: DISCUSS / [REF] User Stories

All stories: `@infrastructure`, `job_id: infrastructure-only`. Build/test **only
via devenv**. **No `std::env::set_current_dir` in tests** — inject via
`CommitSaver::from_repo(&repo)` + `tempfile::tempdir()`.

### US-01: Unify the binary onto the library crate (structural / measurement) `@infrastructure`

- **job_id:** infrastructure-only
- **infrastructure_rationale:** Pure build-structure correction. `src/main.rs`
  re-declares `pub mod vim_commit; pub mod config;` instead of consuming
  `src/lib.rs` (`use rusty_commit_saver::…`), so each module compiles twice and
  llvm-cov averages the tested copy against a dead, never-executed copy. No user
  behavior changes; the diary tool's output is byte-identical before and after.

#### Elevator Pitch

- **Before:** `devenv shell -- test-coverage` reports `vim_commit.rs` at ~45.61%
  (combined) / 43.66% (lib-only), with 33 of 63 function-records showing zero
  executions purely because the module is compiled twice — Sam cannot trust the
  number.
- **After:** Sam runs `devenv shell -- test-coverage` and sees a **single
  instrumented copy** of `vim_commit.rs`; the printed percentage reflects only
  executed code, and `config.rs` (already 99.66% under the same setup) proves the
  gap is now real, not noise.
- **Decision enabled:** Sam can decide which uncovered lines are genuine gaps
  worth a test (Track 2) versus measurement artifacts — impossible while the dead
  copy drags the average.

#### Problem

Sam is the maintainer who reads `devenv shell -- test-coverage` to gate releases.
They find it impossible to act on the 45.61% figure because llvm-cov averages a
tested copy of `vim_commit.rs` against a dead duplicate compiled by `src/main.rs`.

#### Who

- Maintainer / CI pipeline | runs `test-coverage` and the codecov project gate | wants a trustworthy coverage number.

#### Solution

Make `src/main.rs` a thin shim: `use rusty_commit_saver::{vim_commit::…, config::…}`
and drop the duplicate `pub mod vim_commit; pub mod config;` declarations, so there
is a single instrumented copy that the existing tests cover. (Exact mechanics are a
DESIGN-wave decision.)

#### Domain Examples

1. **Happy path (re-measure):** Before, `cargo llvm-cov` lists two compilations of
   `vim_commit.rs`; production fns execute 9–14 times yet 33/63 records read zero.
   After unification, one compilation, zero phantom-zero records.
2. **Reference proof:** `config.rs` reaches 99.66% under the same dual-crate setup
   today — after `vim_commit.rs` is de-duplicated it is measured on equal footing.
3. **Regression guard:** `devenv shell -- cargo test` stays green in parallel and
   the real diary output (a `2025-…/…-…/YYYY-MM-DD.md` table row) is unchanged.

#### UAT Scenarios (BDD)

##### Scenario: Coverage stops double-counting a dead module copy

Given the binary re-declares `pub mod vim_commit` and `pub mod config` alongside the library crate
And `devenv shell -- test-coverage` reports `vim_commit.rs` combined coverage at 45.61%
When Sam makes the binary consume the library crate and removes the duplicate module declarations
Then `devenv shell -- test-coverage` measures a single instrumented copy of `vim_commit.rs`
And no function-record reads zero executions because of duplicate compilation

##### Scenario: Existing behavior and suite stay green after unification

Given the binary now depends only on the library crate's public API
When Sam runs `devenv shell -- cargo test`, then `devenv shell -- lint`, then `devenv shell -- build-release`
Then the full suite passes in parallel
And clippy is clean
And the release build succeeds

##### Scenario: The reported percentage reflects only executed code

Given `config.rs` already reports 99.66% under the dual-crate setup
When the duplicate compilation of `vim_commit.rs` is removed
Then the `vim_commit.rs` percentage printed by `devenv shell -- test-coverage` no longer averages in a never-executed copy

#### Acceptance Criteria

- [ ] `cargo llvm-cov` (via `devenv shell -- test-coverage`) shows exactly one compilation of `vim_commit.rs` and one of `config.rs`.
- [ ] No `vim_commit.rs` function-record reports zero executions caused by duplicate compilation.
- [ ] `devenv shell -- cargo test` passes in parallel; `devenv shell -- lint` clean; `devenv shell -- build-release` green.
- [ ] The diary file output (frontmatter + table row) is unchanged by the refactor.

#### Outcome KPIs

- **Who:** the build system / `cargo llvm-cov`.
- **Does what:** compiles `vim_commit.rs` and `config.rs` once each (no dead duplicate).
- **By how much:** duplicate copies reduced from 2 to **0** per module.
- **Measured by:** function-record inspection under `devenv shell -- test-coverage`.
- **Baseline:** 2 compilations per module; 33/63 phantom-zero records on `vim_commit.rs`.

#### Technical Notes

- Build/test only via devenv (bare shell has clippy 0.1.86 vs rustc 1.94 mismatch).
- After unification, error-branch tests currently living in `main.rs`'s `main_tests`
  (which test the binary's duplicate copy) must count against the single library
  copy — this is the dependency that US-02..04 build on.
- `src/main.rs` is in `.codecov.yml` `ignore:`; the gate measures lib coverage.

---

### US-02: Cover CommitSaver construction error branches `@infrastructure`

- **job_id:** infrastructure-only
- **infrastructure_rationale:** Adds fault-injection unit tests for `# Errors`
  paths in `from_repo` / `try_new` / `default`. No user behavior change — these
  branches already exist; they were simply unmeasured. Pure test-coverage work.

#### Elevator Pitch

- **Before:** `devenv shell -- test-coverage` shows the `?`/`ok_or` error arms of
  `from_repo`, and the discovery failure in `try_new`, as uncovered red lines.
- **After:** Sam runs `devenv shell -- test-coverage` and those construction
  error lines render covered (green), each exercised by a `from_repo(&repo)` test
  against a crafted `tempfile` repository.
- **Decision enabled:** Sam can trust that a malformed repo (no HEAD, detached
  HEAD, bad timestamp) is reported as an error rather than panicking in the field.

#### Problem

Sam cannot confirm that `CommitSaver` construction fails gracefully on broken
repositories, because the `# Errors` branches in `from_repo`/`try_new`/`default`
have no covering test — they show as gaps once US-01 removes the measurement noise.

#### Who

- Maintainer / CI pipeline | reads per-line coverage of `vim_commit.rs` | wants every error arm exercised.

#### Solution

Add fault-injection tests building `git2::Repository` handles in `tempfile::tempdir()`
and calling `CommitSaver::from_repo(&repo)` for: no-HEAD repo, detached HEAD, and
an out-of-representable-range commit timestamp; plus a `try_new` discovery-failure case.

#### Domain Examples

1. **No HEAD:** `Repository::init(tempdir)` with **no commit** → `from_repo` hits
   `git_repo.head()?` error arm and returns `Err`, not a panic.
2. **Detached HEAD:** repo checked out at a bare commit id → `head.shorthand()`
   is `None` → branch records the `"no_branch_set"` placeholder.
3. **Out-of-range timestamp:** a commit whose `Signature` time is beyond chrono's
   representable range → `DateTime::from_timestamp(...).ok_or(...)` error arm fires.

#### UAT Scenarios (BDD)

##### Scenario: A repository with no commit is reported, not panicked

Given a freshly initialized git repository in a tempdir with no HEAD commit
When `CommitSaver::from_repo` is called on it
Then it returns an error
And the process does not panic

##### Scenario: A detached HEAD records a placeholder branch name

Given a tempdir repository checked out at a detached HEAD with one commit
When `CommitSaver::from_repo` is called on it
Then construction succeeds
And the branch name is recorded as `no_branch_set`

##### Scenario: An out-of-range commit timestamp is reported as an error

Given a tempdir repository whose HEAD commit carries a timestamp outside the representable range
When `CommitSaver::from_repo` is called on it
Then it returns the "commit timestamp is out of range" error

##### Scenario: Discovery failure propagates through try_new

Given a directory tree that is not inside any git repository
When `CommitSaver::try_new` runs there (via an injected path, never `set_current_dir`)
Then it returns an error from `Repository::discover`

#### Acceptance Criteria

- [ ] A `from_repo` test against a no-HEAD repo asserts `Err` and covers the `head()?` arm.
- [ ] A `from_repo` test covers the detached-HEAD `no_branch_set` branch.
- [ ] A test covers the out-of-range-timestamp `ok_or` error arm.
- [ ] A `try_new` test covers the `Repository::discover` failure arm without mutating the process CWD.

#### Outcome KPIs

- **Who:** the `vim_commit.rs` test suite.
- **Does what:** covers the construction (`from_repo`/`try_new`/`default`) error branches.
- **By how much:** from ~0 covered to **all 4** enumerated construction arms covered.
- **Measured by:** per-line coverage under `devenv shell -- test-coverage`.
- **Baseline:** error arms uncovered (noise-masked pre-US-01).

#### Technical Notes

- **Depends on US-01** (single instrumented copy).
- Inject repos via `from_repo(&repo)` + `tempfile`; never `std::env::set_current_dir`.
- Out-of-range-timestamp construction may need a crafted `git2::Time`; if it
  proves infeasible in devenv, flag during DISTILL (tracked risk, not a blocker).

---

### US-03: Cover diary filesystem error branches `@infrastructure`

- **job_id:** infrastructure-only
- **infrastructure_rationale:** Adds fault-injection tests for IO `# Errors`
  paths in `append_entry_to_diary`, `create_diary_file`, and
  `create_directories_for_new_entry`. No behavior change; closes measured gaps.

#### Elevator Pitch

- **Before:** `devenv shell -- test-coverage` shows the IO-failure arms of the
  diary-writing functions as uncovered.
- **After:** Sam runs `devenv shell -- test-coverage` and the open/write/mkdir
  failure arms render covered, exercised by `tempfile` paths that are missing or
  read-only.
- **Decision enabled:** Sam can trust that a full disk, missing file, or
  permission-denied vault surfaces an `Err` (logged by `main`) rather than silent
  data loss.

#### Problem

Sam cannot confirm the diary-writing functions fail safely on filesystem errors,
because the IO `# Errors` arms of `append_entry_to_diary`, `create_diary_file`,
and `create_directories_for_new_entry` are uncovered against the single library copy.

#### Who

- Maintainer / CI pipeline | reads per-line coverage | wants IO failure arms exercised.

#### Solution

Add tests using `tempfile::tempdir()` paths: append to a non-existent diary file;
create a diary file under a forbidden/read-only location; create directories under
a path that cannot be created. Assert each returns `Err`.

#### Domain Examples

1. **Append to missing file:** `append_entry_to_diary(&PathBuf::from(".../nope.md"))`
   → `OpenOptions::append(true).open(wiki)?` error arm → `Err`.
2. **Create file in read-only location:** `create_diary_file("/proc/invalid/file.md", …)`
   → `fs::write(...)?` error arm → `Err`.
3. **Create dirs under forbidden path:** `create_directories_for_new_entry(/proc/invalid/path/file.md)`
   → `fs::create_dir_all(...)?` error arm → `Err`.

#### UAT Scenarios (BDD)

##### Scenario: Appending to a missing diary file is reported as an error

Given a diary path inside a tempdir that does not exist on disk
When `append_entry_to_diary` is called with that path
Then it returns an error and writes nothing

##### Scenario: Creating a diary file in an unwritable location is reported as an error

Given a target diary path under a read-only / forbidden location
When `create_diary_file` is called with that path
Then it returns an error

##### Scenario: Creating directories under a forbidden path is reported as an error

Given a diary path whose parent directories cannot be created
When `create_directories_for_new_entry` is called with that path
Then it returns an error

#### Acceptance Criteria

- [ ] A test covers the `OpenOptions::open(wiki)?` failure arm in `append_entry_to_diary`.
- [ ] A test covers the `fs::write(...)?` failure arm in `create_diary_file`.
- [ ] A test covers the `fs::create_dir_all(...)?` failure arm in `create_directories_for_new_entry`.
- [ ] All three live against the single library copy (post US-01), not the binary's `main_tests`.

#### Outcome KPIs

- **Who:** the `vim_commit.rs` test suite.
- **Does what:** covers the filesystem IO error branches.
- **By how much:** all **3** enumerated IO arms covered.
- **Measured by:** per-line coverage under `devenv shell -- test-coverage`.
- **Baseline:** arms covered only against the binary's dead copy (don't count) or uncovered.

#### Technical Notes

- **Depends on US-01.** Existing `main_tests` cases (e.g. `test_create_diary_file_error_handling`,
  `test_create_directories_for_new_entry_invalid_path`) may be relocated/duplicated
  into the library test module so they count toward the single instrumented copy.
- Read-only assertions on `/proc`-style paths are Linux-specific; devenv is Linux — acceptable.

---

### US-04: Cover path-inspection error branches `@infrastructure`

- **job_id:** infrastructure-only
- **infrastructure_rationale:** Adds tests for the boundary `# Errors` arms of
  `check_diary_path_exists` (not found) and `get_parent_from_full_path` (root /
  no parent). No behavior change; closes the last measured gaps to reach ≥95%.

#### Elevator Pitch

- **Before:** `devenv shell -- test-coverage` shows the "path does not exist" and
  "no parent" error arms as uncovered against the library copy.
- **After:** Sam runs `devenv shell -- test-coverage` and both boundary arms
  render covered, pushing `vim_commit.rs` over the codecov 95% project gate.
- **Decision enabled:** Sam can flip the codecov project status from **failing**
  to **passing** and merge with a trustworthy gate.

#### Problem

Sam needs the last boundary error arms — a missing diary path and a parentless
root path — covered against the library copy so `vim_commit.rs` clears the 95%
gate; today they are measured only on the binary's dead duplicate.

#### Who

- Maintainer / CI pipeline | reads the codecov project gate on each PR | wants it green and honest.

#### Solution

Add library-side tests: `check_diary_path_exists` on a non-existent tempdir path
(asserts the `"Path does not exist!"` error) and a happy existing-path case;
`get_parent_from_full_path(Path::new("/"))` (asserts the no-parent error) and a
nested-path happy case.

#### Domain Examples

1. **Missing path:** `check_diary_path_exists(&tempdir.join("nonexistent.md"))` →
   `Err("Path does not exist!")`.
2. **Root has no parent:** `get_parent_from_full_path(Path::new("/"))` → `Err`.
3. **Happy boundary:** `get_parent_from_full_path("/home/user/file.txt")` →
   `Ok("/home/user")`; `check_diary_path_exists` on a created tempfile → `Ok`.

#### UAT Scenarios (BDD)

##### Scenario: Checking a missing diary path reports it does not exist

Given a diary path inside a tempdir that was never created
When `check_diary_path_exists` is called with that path
Then it returns the `"Path does not exist!"` error

##### Scenario: Requesting the parent of the filesystem root reports no parent

Given the filesystem root path `/`
When `get_parent_from_full_path` is called with it
Then it returns the "no parent directory" error

##### Scenario: Normal paths resolve their parent and existence successfully

Given an existing diary file in a tempdir and a nested path `/home/user/file.txt`
When `check_diary_path_exists` and `get_parent_from_full_path` are called respectively
Then existence returns Ok and the parent resolves to `/home/user`

#### Acceptance Criteria

- [ ] A library-side test covers the `check_diary_path_exists` not-found error arm.
- [ ] A library-side test covers the `get_parent_from_full_path` no-parent (root) error arm.
- [ ] Happy-path counterparts confirm the success arms.
- [ ] After this story, `vim_commit.rs` line coverage ≥ 95% under `devenv shell -- test-coverage`.

#### Outcome KPIs

- **Who:** the codecov project gate.
- **Does what:** reports `vim_commit.rs` line coverage.
- **By how much:** **≥ 95%** (from 43.66% lib / 45.61% combined).
- **Measured by:** `devenv shell -- test-coverage` and the codecov project status on the PR.
- **Baseline:** project coverage 57.82%, gate failing.

#### Technical Notes

- **Depends on US-01, US-02, US-03.** This is the closing slice that crosses the gate.
- Existing `main_tests` (`test_check_diary_path_exists_false`, `test_get_parent_from_full_path_root`)
  must be present library-side to count toward the single copy.

---

## Wave: DISCUSS / [REF] Out of Scope

- Refactoring or changing any user-facing diary behavior, output format, or config schema.
- Improving `config.rs` (already 99.66% — left as-is).
- `src/main.rs` coverage (in `.codecov.yml` `ignore:`).
- Property-based testing beyond what fits lean rigor (DISTILL may add if it fits).
- Patch-coverage gate tuning (`patch.target: 90%`) — unchanged.

## Wave: DISCUSS / [REF] Walking Skeleton Strategy

**N/A (D2).** Brownfield, isolated quality feature; the application already runs
end-to-end and ships. There is no new end-to-end user flow to skeletonize. The
equivalent "thinnest first" move is **Slice 01 (structural unification)**, which
re-baselines the measurement before any new test is written.

## Wave: DISCUSS / [REF] Driving Ports (entry points / adapters)

| Driving port | Role | Observable signal |
|---|---|---|
| `devenv shell -- test-coverage` | CI-matching coverage run (`cargo llvm-cov` via nextest) | printed per-file / per-line coverage %; function-records |
| `devenv shell -- coverage-check` | local coverage check | pass/fail vs threshold |
| codecov **project** gate | release gate (`.codecov.yml` `target: 95%`, `threshold: 2%`) | PR status: PASS/FAIL |
| `devenv shell -- cargo test` | parallel suite | green/red |
| `devenv shell -- lint` / `build-release` | clippy + release guardrails | clean / green |

No new application driving ports (no new CLI subcommands) are introduced — this is
measurement + test infrastructure behind existing entry points.

## Wave: DISCUSS / [REF] Pre-requisites

- devenv toolchain available (rustc + clippy paired at 1.97). Bare shell unusable for clippy.
- `cargo llvm-cov` + nextest wired (already present; brief reports measured numbers).
- `tempfile` dev-dependency present (it is, `Cargo.toml` v3.27.0).
- `CommitSaver::from_repo(&repo)` injectable constructor present (it is).

## Wave: DISCUSS / [REF] Outcome KPIs

### Objective

By the end of DELIVER, `src/vim_commit.rs` is measured on a single instrumented
copy and clears the codecov 95% project gate with every enumerated error branch
covered — turning a failing, untrustworthy gate into a passing, honest one.

### Outcome KPIs

| # | Who | Does What | By How Much | Baseline | Measured By | Type |
|---|-----|-----------|-------------|----------|-------------|------|
| 1 | codecov project gate | reports `vim_commit.rs` line coverage | **≥ 95%** | 43.66% lib / 45.61% combined | `devenv shell -- test-coverage` + codecov status | Leading |
| 2 | build system (`cargo llvm-cov`) | compiles `vim_commit`/`config` once | **0** duplicate copies (from 2) | 2 copies/module | function-record inspection | Leading |
| 3 | `vim_commit.rs` test suite | covers enumerated `# Errors` branches | **all** listed arms ≥1 test | ~0 (noise-masked) | per-line coverage | Leading |
| 4 | codecov project status | passes the 95% project gate | **PASS** | FAIL (57.82% overall) | codecov gate on PR | Lagging |

### Metric Hierarchy

- **North Star:** `vim_commit.rs` line coverage ≥ 95% under `devenv shell -- test-coverage`.
- **Leading indicators:** single instrumented copy (KPI 2); each error arm covered (KPI 3).
- **Guardrail metrics (must NOT degrade):** full suite green in parallel; clippy clean;
  release build green; `config.rs` stays ≥ 99%.

### Measurement Plan

| KPI | Data Source | Collection Method | Frequency | Owner |
|-----|------------|-------------------|-----------|-------|
| 1,3 | `cargo llvm-cov` report | `devenv shell -- test-coverage` | per PR / per slice | Sam (maintainer/CI) |
| 2 | llvm-cov function-records | manual inspection once post-US-01 | once (Slice 01) | Sam |
| 4 | codecov | PR status check | per PR | codecov |

## Wave: DISCUSS / [REF] DoR Validation (9-Item Hard Gate)

Applies to all 4 stories (US-01..04). Evidence is shared where identical.

| # | DoR Item | Status | Evidence |
|---|----------|--------|----------|
| 1 | Problem statement clear, domain language | PASS | Each story states Sam's measurement/trust pain in coverage-gate terms |
| 2 | Persona with specific characteristics | PASS | Sam = maintainer/CI, instruments named (test-coverage, codecov project gate) |
| 3 | 3+ domain examples with real data | PASS | Each story has 3 examples with real fns, real %, real paths (`/proc/...`, tempdir) |
| 4 | UAT in Given/When/Then (3-7) | PASS | US-01:3, US-02:4, US-03:3, US-04:3 scenarios |
| 5 | AC derived from UAT | PASS | Each story's AC checklist maps 1:1 to its scenarios |
| 6 | Right-sized (1-3 days, 3-7 scenarios) | PASS | Each ≤1 day, 3-4 scenarios; whole feature ~3 days |
| 7 | Technical notes: constraints/deps | PASS | devenv-only, no `set_current_dir`, `from_repo` injection, slice deps noted |
| 8 | Dependencies resolved or tracked | PASS | US-02..04 depend on US-01 (tracked); toolchain/tempfile pre-reqs present |
| 9 | Outcome KPIs with measurable targets | PASS | Per-story KPI block + feature KPI table with numeric targets + method |
| + | `job_id` present (escape valve) | PASS | All 4 carry `job_id: infrastructure-only` + `infrastructure_rationale` |
| + | Elevator Pitch (Before/After/Decision) | PASS | All 4 reference real devenv commands + observable coverage output |

### DoR Status: PASSED (all 9 items + escape-valve checks)

Open tracked item (not a blocker): out-of-range-timestamp construction (US-02) may
be infeasible to craft via `git2` inside devenv — flagged for DISTILL; if so, that
single arm is documented as unreachable rather than tested.

## Wave: DISCUSS / [REF] Definition of Done (9-Item)

1. All UAT scenarios across US-01..04 pass (green) under `devenv shell -- cargo test`.
2. `vim_commit.rs` line coverage ≥ 95% under `devenv shell -- test-coverage`.
3. Exactly one instrumented compilation of `vim_commit.rs` and `config.rs`.
4. Every enumerated `# Errors` branch has ≥1 covering test (or documented unreachable).
5. `devenv shell -- lint` clean (clippy, 1.97).
6. `devenv shell -- build-release` green.
7. Full suite passes **in parallel** (no `set_current_dir`; no flakiness).
8. codecov **project** gate (target 95%) passes on the PR.
9. No change to user-facing diary output/behavior or config schema (regression-checked).

## Wave: DISCUSS / [REF] Wave Decisions Summary

- DISCUSS executed on the **infrastructure escape-valve** path: JTBD/Phase-1 skipped (D4); SSOT bootstrapped (`docs/product/vision.md`, `docs/product/jobs.yaml`).
- Persona reduced to a one-line maintainer/CI (Sam) per lightweight UX depth (D3); no emotional arc / TUI mockups (justified: infrastructure, no user-facing flow).
- Walking skeleton = N/A (D2); structural Slice 01 serves as the re-baseline-first move.
- Scope PASS (4 stories, 1 context, ~3 days); sliced into 4 elephant-carpaccio briefs.
- All-`@infrastructure` slice composition is **exempt via the escape valve** (documented per slice), not a violation.
- DoR PASSED for all 4 stories; one tracked non-blocking risk (out-of-range timestamp).
- Risk note: DIVERGE artifacts absent (expected — escape-valve feature); not required.
- ask-intelligent triggers evaluated at wave end: **none fired** (internal infrastructure feature, lean density).

---

## Wave: DESIGN / [REF] DDD Decisions

| # | Decision | Verdict | One-Line Rationale |
|---|----------|---------|-------------------|
| DDD-1 | Bounded context split (core logic vs CLI) | Single bounded context: "Commit Diary Persistence" | Monolithic crate; no domain decomposition; single aggregate (CommitSaver) sufficient |
| DDD-2 | Aggregate boundary definition | CommitSaver as aggregate root | Captures one commit's metadata; provides persistence operations; cohesive responsibility |
| DDD-3 | Domain events / event sourcing | Not applicable (no audit trail, no temporal queries) | Simple append-only diary; no event sourcing complexity justified |

---

## Wave: DESIGN / [REF] Component Decomposition

| Component | File Path(s) | Responsibility | Change Type (DESIGN) |
|-----------|--------------|-----------------|---------------------|
| **CommitSaver** aggregate | `src/vim_commit.rs` | Struct + methods: capture Git HEAD metadata; coordinate diary writes | EXTEND (no code changes; tests added in DISTILL) |
| **Diary File Operations** | `src/vim_commit.rs` helper functions | create_diary_file, append_entry_to_diary, create_directories_for_new_entry, check_diary_path_exists, get_parent_from_full_path | EXTEND (no changes; fault-injection tests added DISTILL) |
| **Configuration Management** | `src/config.rs` | Parse INI file (GlobalVars struct); once_cell singleton | EXTEND (no changes; already 99.66% coverage) |
| **Business Logic Orchestrator** | `src/lib.rs` run_commit_saver() | Coordinate discover commit → prepare path → create/append diary | EXTEND (visibility + placement unchanged; stays public lib API) |
| **CLI Driving Adapter** | `src/main.rs` main() + entry point | Initialize env_logger, GlobalVars; call run_commit_saver; log errors | EXTEND → REFACTOR (drop duplicate `pub mod` re-declarations; import from lib instead) |
| **Library Public API** | `src/lib.rs` (module re-exports) | Expose CommitSaver, run_commit_saver, helper fns; delegate to sub-modules | EXTEND (no changes; already exposes all public symbols) |

**Summary:** This feature is **entirely EXTEND / REMOVE-DUPLICATION.** Zero new components created. All changes are structural (eliminating dual compilation) or test additions (fault injection).

---

## Wave: DESIGN / [REF] Driving Ports

| Port Name | Role | Observable Signal | Adapter Technology |
|-----------|------|------------------|-------------------|
| `devenv shell -- test-coverage` | CI-matching coverage instrumentation | Printed per-file %; function-record counts | cargo llvm-cov + nextest |
| `devenv shell -- coverage-check` | Local pre-PR coverage check | PASS/FAIL (95% threshold) | cargo llvm-cov (local) |
| `devenv shell -- cargo test` | Automated test suite execution | Green/red (all tests pass in parallel) | cargo test (nextest runner) |
| `devenv shell -- lint` | Static analysis gate | Clippy warnings/errors | clippy 1.97 (via devenv) |
| `devenv shell -- build-release` | Release binary compilation | Success/failure | cargo release build |
| codecov **project gate** | PR coverage enforcement | PR status: PASS/FAIL (target 95%) | Codecov enforcement (`.codecov.yml` config) |

No new driving ports (CLI subcommands) introduced — this is measurement and test infrastructure.

---

## Wave: DESIGN / [REF] Driven Ports & Adapters

| Port Interface | Responsibility | Adapter Impl | Technology | Container Boundary |
|---|---|---|---|---|
| **Git Discovery** | Discover current Git repository from CWD or injected path | git2::Repository::discover() or injected &Repository | git2 0.21.0 (libgit2-sys) | lib (vim_commit.rs::CommitSaver::try_new / from_repo) |
| **Git Metadata Extraction** | Extract HEAD commit hash, branch name, author signature | git2::Repository::head(), git2::Reference, git2::Commit, git2::Signature | git2 0.21.0 | lib (vim_commit.rs::CommitSaver::new / from_repo) |
| **DateTime Representation** | Format commit timestamp for diary paths and frontmatter | chrono::DateTime, DateTime::from_timestamp, format specs | chrono 0.4.44 | lib (vim_commit.rs::CommitSaver fields + diary ops) |
| **Configuration Read** | Load INI file; populate GlobalVars singleton | configparser crate; once_cell for thread-safe singleton | configparser 3.1.0 + once_cell 1.21.4 | lib (config.rs) |
| **Diary File I/O (Write)** | Create diary files; append entries; create parent directories | std::fs, std::fs::File, std::fs::OpenOptions, std::fs::create_dir_all | Rust std library (sync) | lib (vim_commit.rs diary ops) |
| **Diary File I/O (Check)** | Check if diary path exists on filesystem | std::path::Path::exists() | Rust std library | lib (vim_commit.rs::check_diary_path_exists) |
| **Logging** | Emit structured log events (info, error, debug, warn) | log crate macros (info!, error!, etc.) + env_logger configuration | log 0.4.31 + env_logger 0.11.10 | lib (vim_commit.rs) + bin (main.rs) |

---

## Wave: DESIGN / [REF] Technology Choices

| Layer | Selection | Version | License | Rationale | Alternatives |
|-------|-----------|---------|---------|-----------|---------------|
| **Language** | Rust | 1.97 (devenv) | Apache 2.0 + MIT | Type safety; zero-cost abstractions; robust error handling; strong ecosystem for systems tools | C (unsafe), Go (GC overhead), Python (perf) |
| **Git Integration** | git2 | 0.21.0 | MIT | Pure Rust bindings to libgit2; no external git binary required; robust DateTime/Signature handling; widely used in Rust ecosystem | cargo-scm (alternative), gitoxide (newer, less mature) |
| **Datetime & Formatting** | chrono | 0.4.44 | Apache 2.0 + MIT | Standard Rust datetime library; timezone-aware; format specs for diary paths (%Y/%m-%B/%F.md); proven stable | time crate (newer, less ecosystem adoption) |
| **INI Config Parsing** | configparser | 3.1.0 | MIT | Minimal, zero-dependency parser; matches existing codebase style; simple to maintain | toml (not INI format), serde-yaml (over-engineered), hand-parsing (error-prone) |
| **CLI Framework** | clap | 4.6.1 | Apache 2.0 + MIT | Derive macros; extensible for future subcommands; arg parsing with validation; standard in Rust ecosystem | structopt (superseded by clap), getopts (low-level) |
| **Logging** | log + env_logger | 0.4.31 + 0.11.10 | MIT / MIT | Standard Rust logging stack; env_logger for runtime config (RUST_LOG=info); zero cost if disabled | slog (more complex), tracing (heavier), println! (no filtering) |
| **Home Directory** | dirs | 6.0.0 | Apache 2.0 + MIT | XDG-compliant path resolution; cross-platform (Linux/macOS/Windows); standard library for this purpose | home crate (less maintained), home-rolled (platform-specific bugs) |
| **Test Fixtures** | tempfile | 3.27.0 (dev-dep) | Apache 2.0 + MIT | Fault injection via temp directories; no process CWD mutation (safe for parallel tests); proven stable | fs_extra (different purpose), std::fs::create_temp_file (limited), mktemp crate (deprecated) |
| **Coverage Instrumentation** | cargo llvm-cov | (via devenv) | Apache 2.0 | LLVM-based coverage; function-record reporting (detects dead-copy phantom-zero); accurate; integrated with nextest | tarpaulin (slower), kcov (system-dependent) |
| **Test Parallelism** | nextest | (via devenv) | Apache 2.0 + MIT | Parallel test execution without flakiness; no process-global CWD mutations needed; standard in ecosystem | cargo test (slower, sequential), pytest (Python-only) |

**All OSS. No proprietary dependencies. MIT/Apache 2.0 preferred for permissive licensing.**

---

## Wave: DESIGN / [REF] Decisions Table (Consolidated)

| # | Category | Decision Title | Verdict | Consequence |
|---|----------|-------|---------|------------|
| ARCH-1 | Architectural Style | Hexagonal (Ports & Adapters) + modular monolith | ACCEPTED | Library = core logic + ports; binary = CLI adapter. Testability via dependency injection (CommitSaver::from_repo injects &Repository). Clear separation of concerns. |
| ARCH-2 | Binary/Library Unification | Remove duplicate module declarations from main.rs (ADR-001) | ACCEPTED | Single instrumented compilation per module. Eliminates phantom-zero coverage noise. `run_commit_saver()` stays public in lib.rs; `main()` becomes thin CLI shim. Effort: ~30 min. |
| ARCH-3 | Dependency Inversion | Binary depends on library's public API, not internal module structure | ACCEPTED | Future-proof: internal lib structure can change without affecting binary. Clear contract boundary. |
| TECH-1 | Tech Stack Philosophy | Open-source first; all selections OSS with MIT/Apache 2.0 licenses | ACCEPTED | Full transparency; no vendor lock-in; ecosystem support; community-maintained. |
| TECH-2 | New Dependencies | Zero new dependencies; extend existing stack (git2, chrono, configparser, etc.) | ACCEPTED | Reduces risk; leverages proven, battle-tested ecosystem; simplifies auditing. |
| TEST-1 | Test Injection Strategy | Inject Git repos + temp directories; never mutate process CWD | ACCEPTED | Parallel execution safe; no flakiness; fault injection enabled via tempfile. Constraint: no `std::env::set_current_dir` in tests. |
| MEASURE-1 | Coverage Measurement | Single instrumented lib target (no binary averaging) | ACCEPTED (ADR-001) | Coverage % reflects reality once binary unification done. Enables honest codecov gate. |
| MEASURE-2 | Coverage Instrumentation | cargo llvm-cov (LLVM-based) + nextest (parallel runner) | ACCEPTED | Function-record reporting detects phantom-zero copies. Accurate, CI-matching measurement. |

---

## Wave: DESIGN / [REF] Reuse Analysis Table

**Scope:** Feature vim-commit-coverage (US-01..04) applied to the rusty-commit-saver codebase.

| Component / Module | Existing in Codebase? | Change Type | Rationale | ADR Reference |
|---|---|---|---|---|
| **CommitSaver** struct + fields | ✓ Yes (src/vim_commit.rs) | **EXTEND** | Core aggregate; fully implemented. DESIGN makes no changes; DISTILL adds tests. | ADR-001 (unification) |
| CommitSaver::new() | ✓ Yes | EXTEND | Entry point for current Git repo; already public; stays as-is. | ADR-001 |
| CommitSaver::try_new() | ✓ Yes | EXTEND | Fallback constructor; already public; stays as-is. | ADR-001 |
| CommitSaver::from_repo(&repo) | ✓ Yes | EXTEND | Injected constructor for testing; already public; DISTILL adds tests using it. | ADR-001 |
| CommitSaver::append_entry_to_diary() | ✓ Yes | EXTEND | Core diary-append operation; DISTILL adds IO error tests. | ADR-001 |
| CommitSaver::prepare_path_for_commit() | ✓ Yes | EXTEND | Path formatting; no changes DESIGN. | ADR-001 |
| create_diary_file() | ✓ Yes | EXTEND | Diary template creation; DISTILL adds error-branch tests. | ADR-001 |
| create_directories_for_new_entry() | ✓ Yes | EXTEND | mkdir wrapper; DISTILL adds permission-denied tests. | ADR-001 |
| append_entry_to_diary() | ✓ Yes | EXTEND | File append operation; DISTILL adds error-branch tests. | ADR-001 |
| check_diary_path_exists() | ✓ Yes | EXTEND | Existence check; DISTILL adds missing-path tests. | ADR-001 |
| get_parent_from_full_path() | ✓ Yes | EXTEND | Path parent extraction; DISTILL adds root-path tests. | ADR-001 |
| **GlobalVars** config struct | ✓ Yes (src/config.rs) | **EXTEND** | Config singleton; fully implemented at 99.66% coverage; no changes. | ADR-001 |
| **run_commit_saver()** orchestrator | ✓ Yes (src/lib.rs) | **EXTEND** (placement unchanged) | Public library function; stays in lib.rs post-unification; `main()` calls it via library import. | ADR-001 |
| **Binary entry point** (main()) | ✓ Yes (src/main.rs) | **EXTEND** → **REFACTOR** (REMOVE-DUPLICATION) | **Change:** Drop `pub mod vim_commit; pub mod config;` re-declarations; import from library instead: `use rusty_commit_saver::{vim_commit::…, config::…, run_commit_saver}`. Eliminates dual compilation. | ADR-001 |
| **Library public API** (src/lib.rs mod/use statements) | ✓ Yes | **EXTEND** | Already exposes CommitSaver, run_commit_saver, helpers; no changes DESIGN. | ADR-001 |
| Tests in main_tests module | ✓ Yes (src/main.rs main_tests) | **EXTEND** (may relocate DISTILL) | Existing tests measure against binary's dead copy; post-unification (ADR-001) may move error-path tests to lib for single-copy measurement. Non-breaking refactor. | ADR-001, US-02..04 |

**Summary:**
- **0 new components created**
- **All EXTEND:** Existing logic stays; structural refactoring + test additions
- **Key change:** Binary converts from module-re-declaring to library-importing (REMOVE-DUPLICATION); eliminates dual compilation noise

---

## Wave: DESIGN / [REF] Open Questions (Deferred to DISTILL/DELIVER)

| # | Question | Deferred To | Owner | Risk / Tracking |
|---|----------|------------|-------|-----------------|
| Q1 | Out-of-range timestamp construction: Can `git2::Time` be crafted outside chrono's representable range inside devenv? If infeasible, is this arm unreachable (US-02)? | DISTILL | acceptance-designer (test-crafter) | Non-blocking tracked risk: if infeasible, document arm as unreachable rather than blocked. |
| Q2 | Permission-denied IO errors: Do read-only `/proc`-style assertions work reliably in devenv Linux? | DISTILL | acceptance-designer | Non-blocking: verify permission escapes work reliably; adjust test if needed. |
| Q3 | Regression check: Can diary output be verified byte-identical pre/post refactor? (frontmatter + table row) | DELIVER | software-crafter | Non-blocking: craft reference output before refactor; verify after. |
| Q4 | Module organization post-unification: Are there any edge cases in how Rust module-scoping interacts with library imports in main.rs? | DELIVER | software-crafter | Low risk: straightforward `use` statement; cargo will catch any issues at compile time. |

---

## Wave: DESIGN / [REF] Upstream Changes

**Assumption Changes:** None. DESIGN does not modify DISCUSS assumptions.

**User-Facing Changes:** None. Diary output format (structure, content, filename) unchanged. Configuration schema unchanged. CLI interface unchanged.

**Test Changes:** None at DESIGN level. Fault-injection tests added at DISTILL/DELIVER (US-02..04).

**Measurement Changes:** Post-implementation (DELIVER), coverage measurement will reflect single instrumented copy (no dead-copy averaging). Codecov gate should transition from FAIL to PASS once US-01..04 complete.

---

## Wave: DISTILL / [REF] Test Scenario List & US Mapping

| US | Test Function | Tag | Activation Classification | Rationale |
|----|----|----|---|---|
| US-02 | `test_from_repo_no_head_error` | `@error` | PASS-on-activation | No-HEAD repo error arm already implemented; test validates |
| US-02 | `test_from_repo_detached_head_branch_head` | `@happy` | PASS-on-activation | Detached HEAD behavior correct (returns "HEAD"); test validates actual behavior ≠ spec expectation |
| US-02 | `test_from_repo_out_of_range_timestamp_unreachable` | `@documented-unreachable` | DOCUMENTED-UNREACHABLE | Error arm unreachable: chrono represents full i64 range; see upstream-issues.md |
| US-02 | `test_try_new_discovery_failure_blocked` | `@blocked` | BLOCKED | Path-injection refactor needed; see upstream-issues.md for recommendation |
| US-03 | `test_append_entry_to_diary_parent_not_exists` | `@error` | PASS-on-activation | Missing parent error arm already implemented; test validates |
| US-03 | `test_create_diary_file_unwritable_location` | `@error` | PASS-on-activation | Permission-denied error arm already implemented; test validates |
| US-03 | `test_create_directories_forbidden_path` | `@error` | PASS-on-activation | Mkdir permission error arm already implemented; test validates |
| US-04 | `test_check_diary_path_exists_missing_error` | `@error` | PASS-on-activation | Missing-path error branch already implemented; test validates |
| US-04 | `test_check_diary_path_exists_happy_path` | `@happy` | PASS-on-activation | Existing-path success branch already implemented; test validates |
| US-04 | `test_get_parent_from_full_path_root_error` | `@error` | PASS-on-activation | Root-path error branch already implemented; test validates |
| US-04 | `test_get_parent_from_full_path_nested_happy` | `@happy` | PASS-on-activation | Nested-path success branch already implemented; test validates |

**Total:** 11 tests
**Expected on DELIVER activation:** 9 PASS | 1 DOCUMENTED-UNREACHABLE (leave ignored) | 1 BLOCKED (awaits refactor)

---

## Wave: DISTILL / [REF] Test Placement & Adapter Coverage

### Test Placement

All tests are placed in the **inline `#[cfg(test)] mod commit_saver_tests`** inside `src/vim_commit.rs` (starting ~line 795). This is the library copy (post-US-01 unification), so coverage counts toward the single instrumented compilation. Each test is marked:
```rust
#[test]
#[ignore = "DISTILL scaffold — pending DELIVER activation"]
fn test_name() { ... }
```

**Justification:** US-01 unifies the binary to import from the library, eliminating dual compilation. Placing tests in the library's inline module ensures they measure coverage against the single instrumented copy that both `cargo test` and `cargo llvm-cov` execute.

### Driven Adapter Coverage

| Adapter | Port Type | Technology | Test Coverage | Note |
|---------|-----------|-----------|---|---|
| Git discovery (Repository::discover) | Driven internal | git2 library | `test_try_new_discovery_failure_blocked` (BLOCKED) | Path injection not available; see upstream-issues.md |
| Git metadata extraction (head, commit) | Driven internal | git2 library | `test_from_repo_no_head_error`, `test_from_repo_detached_head_branch_head` | Both error and happy paths covered |
| Filesystem write (fs::write) | Driven internal | Rust std lib | `test_create_diary_file_unwritable_location` | Error path covered (permission denied) |
| Filesystem mkdir (fs::create_dir_all) | Driven internal | Rust std lib | `test_create_directories_forbidden_path` | Error path covered (permission denied) |
| Filesystem append (OpenOptions::open) | Driven internal | Rust std lib | `test_append_entry_to_diary_parent_not_exists` | Error path covered (file not found) |
| Filesystem exists check (Path::exists) | Driven internal | Rust std lib | `test_check_diary_path_exists_missing_error`, `test_check_diary_path_exists_happy_path` | Both error and happy paths covered |
| Filesystem parent extraction (Path::parent) | Driven internal | Rust std lib | `test_get_parent_from_full_path_root_error`, `test_get_parent_from_full_path_nested_happy` | Both error (no parent) and happy paths covered |

---

## Wave: DISTILL / [REF] Scaffolds

No production-code scaffolds created (not applicable to Rust coverage feature; all production code already exists). All acceptance tests are placed in the library's inline test module.

---

## Wave: DISTILL / [REF] Pre-requisites & Dependencies

### Language & Build
- **Language:** Rust 1.97 (via devenv)
- **Build tool:** cargo
- **Test framework:** std `#[test]` macros (cargo test)
- **Compile verification:** `devenv shell -- cargo test --no-run` ✓ PASS

### Dev Dependencies (already present)
- `tempfile 3.27.0` — fault injection via temp directories (used in all tests)
- `git2 0.21.0` — git repository creation and manipulation

### Test Execution Environment
- **Test runner:** `devenv shell -- cargo test --lib -- --ignored`
- **Parallel execution:** Safe (no `std::env::set_current_dir` mutations)
- **Coverage measurement:** `devenv shell -- test-coverage` (cargo llvm-cov + nextest)

### Verification Status
- ✓ All tests compile
- ✓ All 11 tests pass when run with `--ignored`
- ✓ Zero REAL-BUG findings
- ✓ Two behavior discoveries documented in `upstream-issues.md` (detached HEAD actual behavior, timestamp unreachability)
- ✓ One blocked test documented for DELIVER path-injection refactor

---

## Wave: DISTILL / [REF] Findings & Issues

### Upstream Issues Identified

See `docs/feature/vim-commit-coverage/distill/upstream-issues.md` for full detail.

| Issue | Component | Severity | Status | Action |
|-------|-----------|----------|--------|--------|
| 1. Path injection not available | `CommitSaver::try_new` | Medium | BLOCKED | Recommend `try_discover` helper in DELIVER |
| 2. Timestamp error unreachable | `CommitSaver::from_repo` | Low | DOCUMENTED-UNREACHABLE | Inline comment; accept 0 coverage for this arm |
| 3. Detached HEAD returns "HEAD" | `CommitSaver::from_repo` | Informational | DISCOVERY | Spec expectation was incomplete; actual behavior correct |

### Test Activation Classification

See `docs/feature/vim-commit-coverage/distill/red-classification.md` for full detail.

- **PASS-on-activation:** 9 tests (behavior already implemented)
- **DOCUMENTED-UNREACHABLE:** 1 test (error arm unreachable; defensive code)
- **BLOCKED:** 1 test (awaits path-injection refactor)

---

## Wave: DISTILL / [REF] Definition of Done Checklist

- [x] All 11 acceptance tests written and compile successfully
- [x] All tests marked `#[ignore = "DISTILL scaffold — pending DELIVER activation"]`
- [x] All tests placed in library's inline `mod commit_saver_tests` (post-US-01 single instrumented copy)
- [x] Compile verification: `cargo test --no-run` ✓
- [x] Activation verification: all tests pass when run with `--ignored` ✓
- [x] Real-bug findings: zero (no production bugs exposed)
- [x] Behavior discoveries documented: 2 (detached HEAD, timestamp unreachability)
- [x] Upstream issues documented: 3 (path injection, timestamp unreachable, behavior discovery)
- [x] Red-classification performed: 9 PASS-on-activation, 1 DOCUMENTED-UNREACHABLE, 1 BLOCKED
- [x] Test scenarios mapped 1:1 to US-02/03/04 domains
- [x] Driven adapters covered: git2, fs write/mkdir/append/exists/parent
- [x] Parallel execution safe: no CWD mutations, only tempfile isolation
- [x] Feature-delta.md updated with DISTILL [REF] sections
- [x] `distill/upstream-issues.md` created with findings
- [x] `distill/red-classification.md` created with activation plan

---

## Wave: DISTILL | Self-Review Checklist

**Tier-1 [REF] Coverage (Lean Mode)**

- [x] Scenario list with tags (US → test mapping) — Table above ✓
- [x] Test placement rationale — `src/vim_commit.rs` inline module, post-US-01 unification ✓
- [x] Driving adapter coverage — 7 adapters (git2, fs) with happy + error paths ✓
- [x] Pre-requisites — devenv, tempfile, git2; all verified ✓
- [x] Upstream issues flagged — 3 items: path injection, timestamp unreachable, behavior discovery ✓
- [x] Activation classification — 9 PASS, 1 UNREACHABLE, 1 BLOCKED ✓

**Rust-Specific Mandates**

- [x] No `std::env::set_current_dir` (parallel-safe) ✓
- [x] Injection via `CommitSaver::from_repo(&repo)` + `tempfile::tempdir()` ✓
- [x] All tests use devenv-only cargo commands ✓
- [x] Compile verification via `cargo test --no-run` ✓
- [x] Run verification via `cargo test --lib -- --ignored` ✓

**Lean Mode Appropriateness** — This is a **coverage feature** (existing code, fault-injection tests). Tier-1 [REF] is appropriate; no Tier-2 expansions needed. Feature scope is narrow (error branches, 4 user stories), so Tier-2 would be over-documentation.

---

## Wave: DELIVER / [REF] Implementation Summary

Raised `src/vim_commit.rs` line coverage **54.21% → 97.66%** (codecov 95% project gate met). The decisive fix was a one-line annotation — `#[cfg_attr(coverage_nightly, coverage(off))]` on the inline `mod commit_saver_tests` — matching the project's existing convention in `config.rs`/`main.rs`; `vim_commit.rs` was the only module missing it, so its 456-line test module was counted in the denominator. Plus: the ADR-001 lib/bin unification (binary now consumes the library crate), a `try_discover(path)` helper making the discover-failure arm testable, and activation of the 11 DISTILL error-branch tests.

## Wave: DELIVER / [REF] Files Modified

| File | Kind | Change |
|------|------|--------|
| `src/vim_commit.rs` | production + tests | `coverage(off)` on test module; `try_discover` helper; `try_new` delegates to it; earlier `from_repo`/`try_new` refactor; 11 activated error-branch tests |
| `src/main.rs` | production | consumes `rusty_commit_saver::{vim_commit, config}` (no `pub mod` re-decl) |
| `docs/product/architecture/*`, `docs/feature/vim-commit-coverage/*` | docs | wave artifacts + DES roadmap/execution-log |

## Wave: DELIVER / [REF] Scenarios Green

107 tests pass / 1 ignored (out-of-range-timestamp, documented-unreachable) / 0 failed — parallel, 2026-06-20.

## Wave: DELIVER / [REF] DoD Check

- [x] All UAT scenarios green (`cargo test`) · [x] vim_commit.rs ≥95% (97.66%) · [x] single instrumented compilation · [x] every enumerated error branch covered or documented-unreachable · [x] clippy clean · [x] release green · [x] parallel suite, no `set_current_dir` · [x] no user-facing behavior change. Codecov **project gate**: confirm on PR (local metric 97.66% > 95%).

## Wave: DELIVER / [REF] Quality Gates

Refactor (Phase 3), adversarial review (Phase 4), mutation (Phase 5) — **skipped per lean rigor**. DES integrity (Phase 6): **PASS** (2 steps, complete traces). Demo gate (Phase 3.5): N/A (all stories `@infrastructure`).

## Wave: DELIVER / [WHY] Upstream Issues / Deviations

- **DESIGN ADR-001 mechanic #3 deviation:** ADR assumed `run_commit_saver` lived in `lib.rs`; it lives in `main.rs`. Left in place (codecov ignores `main.rs`; moving it was unnecessary for the goal).
- **Root-cause correction:** the coverage gap was NOT dominated by lib/bin dual-compilation (the DESIGN focus) but by the missing test-module `coverage(off)` annotation. See `docs/evolution/vim-commit-coverage-evolution.md` retrospective.
