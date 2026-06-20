# Wave: DESIGN — Decisions & Architecture Summary

**Feature:** vim-commit-coverage
**Wave:** DESIGN (Application/components scope, Propose mode)
**Date:** 2026-06-20
**Architect:** Morgan (Solution Architect)

---

## Key Decisions

### DDD-1: Bounded Context (Accepted)
**Verdict:** Single bounded context: "Commit Diary Persistence"
**Rationale:** Monolithic crate with a single responsibility: capture Git commit metadata and append to Obsidian diary files. No domain-driven decomposition needed; single aggregate (CommitSaver) with value objects (metadata fields). No subdomain split.

### DDD-2: Aggregate Boundary (Accepted)
**Verdict:** CommitSaver is the aggregate root; no further decomposition into domain entities.
**Rationale:** CommitSaver captures a single commit's metadata and provides operations to persist it. Simple, cohesive. No transactional boundary beyond individual diary appends.

### ARCH-1: Architectural Style (Accepted)
**Verdict:** Hexagonal Architecture (Ports & Adapters) with modular monolith.
**Rationale:** Clear separation: library = core logic (business logic, ports); binary = driving adapter (CLI entry point). Testability via dependency injection (CommitSaver::from_repo accepts &Repository). Flexibility to swap adapters (filesystem I/O, git integration) without affecting core.

### ARCH-2: Unification Strategy (Accepted — ADR-001)
**Verdict:** Binary becomes a thin CLI shim over the library. Remove duplicate module declarations from main.rs.
**Rationale:** Eliminates dual compilation causing coverage measurement noise. Single instrumented copy enables honest measurement and fault-injection testing. Aligns with hexagonal style.
**Consequence:** `run_commit_saver()` (business logic) stays in lib.rs as public function; `main()` becomes CLI initialization shim. See ADR-001 for full analysis.

---

## Architecture Summary

### Component Boundaries

| Component | Boundary | Responsibility | Compile Target |
|-----------|----------|-----------------|-----------------|
| **CommitSaver** aggregate | `src/vim_commit.rs` | Capture Git metadata; orchestrate diary operations | lib (single copy post-unification) |
| **Diary Operations** | `src/vim_commit.rs` functions | create_diary_file, append_entry_to_diary, path checks | lib |
| **Config Management** | `src/config.rs` | Parse INI; manage global state (GlobalVars) | lib |
| **Business Logic Orchestrator** | `src/lib.rs` fn run_commit_saver | Coordinate core operations (discover → prepare path → create/append) | lib (public API) |
| **CLI Driving Adapter** | `src/main.rs` | Initialize logger, config, call run_commit_saver | bin (shim; imports from lib) |

### Reuse Analysis Table

**Scope:** Feature vim-commit-coverage (US-01..04)

| Component | Existing? | Change Type | Rationale |
|-----------|-----------|-------------|-----------|
| `CommitSaver` struct + methods | ✓ Yes | EXTEND (no code changes DESIGN; tests added DISTILL) | Core logic already complete; needs test coverage only |
| Diary operations (create/append/check) | ✓ Yes | EXTEND | Functions already exist; fault-injection tests added DISTILL |
| Config parsing | ✓ Yes | EXTEND | Already at 99.66% coverage; no changes needed |
| `run_commit_saver()` orchestrator | ✓ Yes | EXTEND (visibility + placement unchanged) | Already public in lib.rs; stays there post-unification |
| Binary entry point (main.rs) | ✓ Yes | EXTEND / REMOVE-DUPLICATION (refactor: drop `pub mod` re-decls, import from lib) | Eliminate dual compilation; convert to thin CLI shim |
| Library re-export (lib.rs) | ✓ Yes | EXTEND | Public API already exposes all needed symbols; no changes |

**Verdict:** This feature is **entirely EXTEND/REMOVE-DUPLICATION.** Zero new components created. All changes are structural refactoring or test additions.

---

## Technology Stack

| Layer | Selection | Version | License | Rationale |
|-------|-----------|---------|---------|-----------|
| **Language** | Rust | 1.97 (devenv) | Apache 2.0 + MIT | Type safety, zero-cost abstractions, ecosystem support |
| **Git Integration** | git2 | 0.21.0 | MIT | Pure Rust; no external git binary; robust Signature/DateTime handling |
| **DateTime** | chrono | 0.4.44 | Apache 2.0 + MIT | Standard library; timezone-aware; format specs for diary dates |
| **Config Parsing** | configparser | 3.1.0 | MIT | Minimal INI parser; OSS; no external dependencies |
| **CLI Framework** | clap | 4.6.1 | Apache 2.0 + MIT | Derive macro builder; extensible for future subcommands |
| **Logging** | log + env_logger | 0.4.31 + 0.11.10 | MIT / MIT | Standard Rust logging; zero-cost at build time if disabled |
| **Path Resolution** | dirs | 6.0.0 | Apache 2.0 + MIT | XDG-compliant home dir; cross-platform |
| **Test Fixtures** | tempfile | 3.27.0 (dev) | Apache 2.0 + MIT | Fault injection via temp dirs; no process CWD mutation |
| **Coverage Instrumentation** | cargo llvm-cov | (devenv) | Apache 2.0 | LLVM-based coverage; accurate function-record reporting |
| **Test Runner** | nextest | (devenv) | Apache 2.0 + MIT | Parallel execution; no flakiness from process CWD |

**OSS First:** All selections are open-source. No proprietary dependencies. MIT/Apache 2.0 preferred for permissive licensing.

---

## Constraints & Assumptions

### Build Constraints
- **Toolchain:** Rust 1.97 via devenv (bare shell has clippy/rustc mismatch; devenv pairs 1.97)
- **Test Execution:** ONLY via `devenv shell -- cargo test` (no `std::env::set_current_dir` in tests; inject via `CommitSaver::from_repo(&repo)` + tempfile)
- **Coverage Measurement:** `devenv shell -- test-coverage` (cargo llvm-cov + nextest)

### Architectural Constraints
- **No Duplication:** Binary must not re-declare library modules (enforced by ADR-001 unification)
- **Single Source of Truth:** vim_commit.rs and config.rs compile once (lib target only)
- **Error Handling:** Result-based; no panic on recoverable faults (IO, git discovery)
- **Observability:** Structured logging via log crate; no silent errors

### Quality Constraints
- **Coverage Target:** vim_commit.rs ≥ 95% line coverage (codecov project gate)
- **Parallel Execution:** Tests must not mutate process-global state (no CWD changes)
- **Regression:** Diary output format (frontmatter + table rows) unchanged by refactor

---

## Decisions Table

| # | Decision | Verdict | Consequence |
|---|----------|---------|-------------|
| DDD-1 | Bounded context decomposition | Single "Commit Diary Persistence" | No subdomain split; one aggregate (CommitSaver) |
| DDD-2 | Aggregate boundary | CommitSaver root; metadata as value objects | Simple transactional boundary (one diary append = one unit of work) |
| ARCH-1 | Architectural style | Hexagonal (ports & adapters) + modular monolith | Library = core; binary = CLI adapter; dependency inversion |
| ARCH-2 | Binary/library unification | Remove duplicate module declarations (ADR-001) | Single instrumented copy; honest measurement; enables fault-injection tests |
| TECH-1 | OSS preference | All selections OSS (MIT/Apache 2.0) | No proprietary dependencies; full transparency |
| TECH-2 | No new dependencies | Extend existing stack (git2, chrono, etc.) | Reduce risk; leverage proven ecosystem |

---

## Driving Ports & Adapters

### Primary Ports (Inbound)

| Port | Adapter | Observable Signal |
|------|---------|------------------|
| `devenv shell -- test-coverage` | cargo llvm-cov + nextest | Printed per-file coverage % |
| `devenv shell -- coverage-check` | cargo llvm-cov (local) | PASS/FAIL vs 95% threshold |
| `devenv shell -- cargo test` | cargo test runner (nextest) | Green/red test results |
| codecov project gate | Codecov enforcement (`.codecov.yml`) | PR status: PASS/FAIL |

No new CLI subcommands introduced (infrastructure feature).

### Secondary Ports (Outbound) & Adapters

| Port Interface | Adapter Implementation | Technology |
|----------------|------------------------|-----------|
| Git discovery | git2::Repository::discover() | git2 library (libgit2-sys bindings) |
| Git metadata extraction | git2::Repository::head(), git2::Commit, git2::Signature | git2 (commit hash, branch name, timestamp) |
| Diary file I/O | std::fs + std::fs::OpenOptions | Rust standard library (sync write) |
| Configuration read | configparser + once_cell | INI file at ~/.config/rusty-commit-saver/*.ini |
| Logging | log crate macros (info!, error!, debug!) | env_logger (runtime configuration via RUST_LOG) |

---

## Open Questions (Deferred to DISTILL/DELIVER)

| # | Question | Deferred To | Note |
|---|----------|------------|------|
| 1 | Out-of-range timestamp construction feasibility | DISTILL | Can `git2::Time` be crafted outside representable chrono range inside devenv? If infeasible, document that arm as unreachable (in US-02). |
| 2 | Test coverage for permission-denied IO errors | DISTILL | `/proc`-style read-only paths are Linux-specific; devenv is Linux. Verify permission assertions work reliably. |
| 3 | Diary output format regression check | DELIVER | Craft a reference diary entry pre-refactor; verify post-refactor output is byte-identical. |

---

## Wave Handoff

**From:** business-analyst (DISCUSS wave)
**To:** solution-architect-reviewer (peer review — **skipped in lean rigor**)
**Next:** acceptance-designer (DISTILL wave) — author fault-injection test code

**Artifacts:**
- Application Architecture: `docs/product/architecture/brief.md`
- ADRs: `docs/product/architecture/adr-001-lib-bin-unification.md`
- C4 Diagrams: Embedded in brief.md (System Context + Container, Mermaid)
- Feature Design: This file (`docs/feature/vim-commit-coverage/design/wave-decisions.md`)

**Upstream Changes:** None. DESIGN does not change DISCUSS assumptions or user-facing behavior.
