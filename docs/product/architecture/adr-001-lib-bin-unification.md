# ADR-001: Library/Binary Unification — Eliminate Duplicate Module Compilation

**Status:** ACCEPTED (DESIGN wave, 2026-06-20)

**Scope:** Application architecture / component boundaries (DESIGN wave)

**Stakeholder:** Sam (maintainer / CI), codecov project gate

---

## Context

The `rusty-commit-saver` crate has both a library target (`src/lib.rs`) and a binary target (`src/main.rs`). Both targets re-declare the same public modules:

```rust
// src/lib.rs
pub mod vim_commit;
pub mod config;

// src/main.rs (OLD)
pub mod vim_commit;       // ← duplicate
pub mod config;           // ← duplicate
```

This causes **dual compilation**: each module is compiled twice (once for lib, once for bin). The binary's compiled copy is never executed in production; only the library's copy is tested and used.

**Measurement Impact:**
- `cargo llvm-cov` instruments and averages coverage across both compilations
- `vim_commit.rs` reports ~44% coverage (lib-only: 43.66%, combined: 45.61%)
- Actual analysis: every function in `vim_commit.rs` executes 9–14 times; yet 33/63 function-records report zero (phantom-zero noise from the dead binary copy)
- `config.rs` reaches 99.66% under the same dual-compilation setup, proving ≥95% is achievable

**Business Impact:**
- Codecov project gate (target 95%) is failing (measured 57.82% overall)
- Sam cannot distinguish real coverage gaps from measurement artifacts
- US-02..04 (error-branch fault-injection tests) cannot close gaps until the dead copy is eliminated

---

## Decision

Convert the binary into a **thin CLI driving adapter** over the library core.

**Mechanics:**

1. **Remove duplicate module declarations** from `src/main.rs`:
   ```rust
   // src/main.rs (OLD)
   pub mod vim_commit;
   pub mod config;

   // src/main.rs (NEW)
   // (removed)
   ```

2. **Import from library** instead:
   ```rust
   // src/main.rs (NEW)
   use rusty_commit_saver::{
       vim_commit::{CommitSaver, check_diary_path_exists, create_diary_file, create_directories_for_new_entry, append_entry_to_diary},
       config::GlobalVars,
       run_commit_saver,
   };
   ```

3. **Sub-decision: `run_commit_saver()` placement:**
   - **Location:** Remains in `src/lib.rs` as a public function (business logic orchestrator)
   - **Rationale:** `run_commit_saver()` is the core orchestration (discover commit → prepare path → create diary → append entry). It's already public in the library and called from `main()`. Moving it back to `main.rs` would re-create duplication or break the library's public API. Keeping it in the library keeps it measurable (codecov instruments the lib, not `main.rs`) and reusable (future users or tests can call it directly).
   - **Impact:** `main()` becomes a thin CLI shim: initialize env_logger + GlobalVars → call `run_commit_saver()` → log errors. The business logic is decoupled from CLI concerns.

**Architectural Style:**
- **Hexagonal Architecture:** Library = core logic (business logic, ports); Binary = driven adapter (CLI entry point)
- **Dependency Inversion:** `main.rs` depends on library abstractions (public API), not on internal module structure
- **Single Source of Truth:** `vim_commit.rs` and `config.rs` compile once; their single instrumented copy is tested and measured

---

## Alternatives Considered

### Alternative A: Lib/Bin Unification (SELECTED)

**Proposal:** As above. Binary imports from library; removes duplicate module declarations.

**Pros:**
- ✅ Single instrumented compilation per module → coverage % reflects reality
- ✅ Aligns with hexagonal architecture (library = core, binary = adapter)
- ✅ Clear dependency inversion (binary depends on library's public API, not internal structure)
- ✅ Enables US-02..04 to close real gaps (fault-injection tests) with confidence
- ✅ Zero user-facing changes; diary output identical before/after
- ✅ Effort: ~30 minutes (DELIVER; non-breaking refactor)
- ✅ Future maintainers see intent clearly: one crate, two targets, clear separation of concerns

**Cons:**
- ⚠ Requires refactoring `main.rs` (struct-based approach stays OOP/imperative; no paradigm shift)
- ⚠ If binary needs internal-only modules in future, must add them to `main.rs`; cannot re-declare library modules

**Trade-off:** Small structural effort for permanent clarity and honest measurement. Recommended.

---

### Alternative B: Measurement-Only Workaround (REJECTED)

**Proposal:** Leave the dual-compilation intact; constrain coverage measurement to `--lib` target or codecov-ignore the binary.

**Implementation:**
- Option B1: Run `cargo llvm-cov --lib` in CI (ignore binary entirely)
- Option B2: Add `.codecov.yml` rule to `ignore` the binary's duplicate compilations (if possible; may require codecov config changes)

**Pros:**
- ✅ Zero code changes
- ✅ Quick (< 5 minutes)
- ✅ Can proceed immediately without DELIVER phase

**Cons:**
- ❌ **Two sources of truth remain:** Library and binary both declare `pub mod vim_commit; pub mod config`. Future maintainers must maintain both; inconsistency risk.
- ❌ **Measurement bypasses reality:** We're hiding a structural problem, not solving it. Codecov interprets the workaround as "lib-only matters," yet the binary ships to users and will receive changes.
- ❌ **Knowledge loss:** Future developer unaware of the flag will run `cargo llvm-cov` (default) and wonder why binary coverage is 0%.
- ❌ **Technical debt:** Violates principle that architecture should express intent clearly. The binary's module re-declarations remain a mystery.
- ❌ **Risk if measurement is forgotten:** If a contributor forgets the `--lib` flag, coverage mysteriously drops again; confusion ensues.

**Why Rejected:** Measurement workaround only; doesn't solve the underlying problem. Leaves technical debt and reduces long-term maintainability. Violates "fix the source" principle.

---

### Alternative C: Status Quo + More Tests (REJECTED)

**Proposal:** Keep dual-compilation; write more tests targeting both the tested and dead copies to increase coverage percentage.

**Implementation:**
- Add fault-injection tests for the binary's copy (duplicate of library tests)
- Attempt to measure both copies equally

**Pros:**
- ✅ Addresses some of the real coverage gaps (error branches do need tests)

**Cons:**
- ❌ **Impossible to reach ≥95%:** Even with 100% testing of the tested copy, the dead binary copy's zero executions average into the percentage. If vim_commit.rs has N lines, tested-copy covers M lines (say 95%), binary-copy covers 0 lines. Average = (M+0)/(N+N) = M/(2N) = 47.5% regardless of M. Mathematically unreachable.
- ❌ **Wasted effort:** Tests are written against a copy that never runs in production. Effort → redundancy, not clarity.
- ❌ **Violates Occam's Razor:** More complex solution (write duplicate tests) instead of simpler fix (unify compilation). Introduces maintenance burden.
- ❌ **Future regression:** If binary's copy is ever removed without removing its tests, tests become dead code.

**Why Rejected:** Root cause not addressed. Cannot mathematically reach ≥95% without Alternative A first. Duplicate tests are waste.

---

## Consequences

### Positive

1. **Honest Measurement:** Single instrumented copy → coverage % reflects reality (no dead-copy averaging)
2. **Enables Fault-Injection Tests:** US-02..04 can now close real error-branch gaps (constructor errors, IO errors, path checks) with confidence
3. **Clear Architecture:** Library is core logic; binary is CLI adapter. Dependency inversion is explicit. Future maintainers see intent.
4. **Non-Breaking:** Public API of library unchanged; diary output unchanged. Existing tests pass without modification.
5. **Measurement Honesty:** Codecov project gate (target 95%) can now legitimately pass (rather than averaging in a dead copy)

### Negative

1. **Refactor Cost:** ~30 min effort in DELIVER (small, but not zero)
2. **Future Module Additions:** If binary needs internal-only modules, cannot re-declare library modules. Must explicitly distinguish in `main.rs` (e.g., `mod bin_only_module;` alongside library imports). Slightly more bookkeeping.

### Risks & Mitigation

| Risk | Mitigation |
|------|-----------|
| Regression in functionality (main.rs no longer compiles modules) | Acceptance tests (UAT scenarios in feature-delta.md); `devenv shell -- cargo test` covers this; diary output is byte-identical before/after |
| Breaking change to library public API | Library public API unchanged (run_commit_saver, CommitSaver, helpers all stay pub); only main.rs imports differ |

---

## Implementation Notes

**Phase:** DELIVER (DESIGN produces this decision; implementation is software-crafter's responsibility)

**Estimated Effort:** ~30 minutes (non-breaking refactor)

**Testing Strategy:**
- Existing tests require no changes; they measure the single instrumented copy
- `devenv shell -- cargo test` must pass in parallel
- `devenv shell -- lint` must pass (clippy 1.97)
- `devenv shell -- test-coverage` must show exactly 1 compilation per module (inspectable via llvm-cov output)
- Regression check: diary file output (frontmatter + table row) is unchanged

**Constraints:**
- Build/test ONLY via devenv (bare shell has clippy/rustc mismatch)
- No `std::env::set_current_dir` in tests (process-global, races parallel tests); use `CommitSaver::from_repo(&repo)` + `tempfile` instead

---

## Related Decisions

- **Feature Scope:** `docs/feature/vim-commit-coverage/feature-delta.md` (US-01 structural unification depends on this ADR)
- **US-02..04:** Error-branch fault-injection tests (depend on ADR-001 to eliminate dead-copy noise)
- **Codecov Gate:** `.codecov.yml` target 95% (currently failing; will pass post-unification + US-02..04)

---

## Audit Trail

| Date | Phase | Decision | Rationale |
|------|-------|----------|-----------|
| 2026-06-20 | DESIGN | Accept Alternative A | Single source of truth; honest measurement; aligns with hexagonal architecture |
