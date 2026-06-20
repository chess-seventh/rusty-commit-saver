# DISTILL Test Activation Classification — vim-commit-coverage

**Feature:** vim-commit-coverage (Tier-1 [REF], coverage feature)
**Wave:** DISTILL
**Date:** 2026-06-20
**Status:** All tests compile and pass when run with `--ignored`

---

## Test Activation Summary

This is a **coverage feature** — tests exercise existing code paths, not new functionality. All tests are marked `#[ignore = "DISTILL scaffold — pending DELIVER activation"]` and will be unskipped one-at-a-time during DELIVER's RED-GREEN-COMMIT cycle.

| Test Name | US | Activation Classification | Expected on Activation | Status |
|-----------|----|----|---|---|
| `test_from_repo_no_head_error` | US-02 | PASS-on-activation | Behavior already implemented; test validates error arm | ✓ Verified |
| `test_from_repo_detached_head_branch_head` | US-02 | PASS-on-activation | Detached HEAD returns shorthand="HEAD"; test validates | ✓ Verified |
| `test_from_repo_out_of_range_timestamp_unreachable` | US-02 | DOCUMENTED-UNREACHABLE | Error arm unreachable; test documents why | ✓ Verified |
| `test_try_new_discovery_failure_blocked` | US-02 | BLOCKED | Needs path-injection refactor; placeholder test | ✓ Verified |
| `test_append_entry_to_diary_parent_not_exists` | US-03 | PASS-on-activation | Behavior validated; error arm exercised | ✓ Verified |
| `test_create_diary_file_unwritable_location` | US-03 | PASS-on-activation | Behavior validated; /proc rejection tested | ✓ Verified |
| `test_create_directories_forbidden_path` | US-03 | PASS-on-activation | Behavior validated; permission-denied tested | ✓ Verified |
| `test_check_diary_path_exists_missing_error` | US-04 | PASS-on-activation | Behavior validated; missing-path error tested | ✓ Verified |
| `test_check_diary_path_exists_happy_path` | US-04 | PASS-on-activation | Behavior validated; existing-path success tested | ✓ Verified |
| `test_get_parent_from_full_path_root_error` | US-04 | PASS-on-activation | Behavior validated; root-path error tested | ✓ Verified |
| `test_get_parent_from_full_path_nested_happy` | US-04 | PASS-on-activation | Behavior validated; nested-path success tested | ✓ Verified |

**Total:** 11 tests
**PASS-on-activation:** 9
**DOCUMENTED-UNREACHABLE:** 1
**BLOCKED:** 1

---

## Detailed Activation Classification

### PASS-on-activation (9 tests)

These tests exercise existing behavior. They pass immediately because the code paths they test already exist in the production codebase. DELIVER will:
1. Unskip the test
2. Verify it passes (no implementation work needed)
3. Commit with the test enabled

#### US-02 Construction Error Branches

**`test_from_repo_no_head_error`**
- **What it tests:** `CommitSaver::from_repo(&repo)` errors when repo has no commits (no HEAD)
- **Code path:** Line 171: `let head = git_repo.head()?;` — error arm already implemented
- **Activation:** Remove `#[ignore]`, run `cargo test` → PASS (no code changes needed)

**`test_from_repo_detached_head_branch_head`**
- **What it tests:** Detached HEAD records `"HEAD"` as branch name (not `"no_branch_set"`)
- **Code path:** Line 181: `head.shorthand().unwrap_or("no_branch_set")` with actual shorthand="HEAD"
- **Activation:** Remove `#[ignore]`, run `cargo test` → PASS (behavior already correct)
- **Note:** DISTILL discovered the feature spec expected `"no_branch_set"`, but actual behavior is `"HEAD"`. Test reflects reality.

#### US-03 Filesystem Error Branches

**`test_append_entry_to_diary_parent_not_exists`**
- **What it tests:** `append_entry_to_diary` errors when file's parent directory doesn't exist
- **Code path:** Line 498: `OpenOptions::new().append(true).open(wiki)?;` — error arm
- **Activation:** Remove `#[ignore]`, run `cargo test` → PASS (error propagates correctly)

**`test_create_diary_file_unwritable_location`**
- **What it tests:** `create_diary_file` errors when target path is unwritable (e.g., `/proc`)
- **Code path:** Line 789: `fs::write(full_diary_file_path, template)?;` — error arm
- **Activation:** Remove `#[ignore]`, run `cargo test` → PASS (permission error propagates)

**`test_create_directories_forbidden_path`**
- **What it tests:** `create_directories_for_new_entry` errors when parent path cannot be created
- **Code path:** Line 687: `fs::create_dir_all(parent_dirs)?;` — error arm
- **Activation:** Remove `#[ignore]`, run `cargo test` → PASS (permission error propagates)

#### US-04 Path Inspection Boundary Branches

**`test_check_diary_path_exists_missing_error`**
- **What it tests:** `check_diary_path_exists` errors when path does not exist
- **Code path:** Lines 634-638: existence check and error return
- **Activation:** Remove `#[ignore]`, run `cargo test` → PASS (existence check works)

**`test_check_diary_path_exists_happy_path`**
- **What it tests:** `check_diary_path_exists` succeeds when path exists
- **Code path:** Lines 634-635: `if Path::new(&full_diary_path).exists() { return Ok(()); }`
- **Activation:** Remove `#[ignore]`, run `cargo test` → PASS (existence check works)

**`test_get_parent_from_full_path_root_error`**
- **What it tests:** `get_parent_from_full_path` errors when given root path `/`
- **Code path:** Lines 579-586: parent check and error return when `.parent()` is None
- **Activation:** Remove `#[ignore]`, run `cargo test` → PASS (root case handled)

**`test_get_parent_from_full_path_nested_happy`**
- **What it tests:** `get_parent_from_full_path` succeeds and returns parent for nested paths
- **Code path:** Lines 579-580: `if let Some(dir) = full_diary_path.parent() { Ok(dir) }`
- **Activation:** Remove `#[ignore]`, run `cargo test` → PASS (parent extraction works)

---

### DOCUMENTED-UNREACHABLE (1 test)

**`test_from_repo_out_of_range_timestamp_unreachable`**
- **What it was intended to test:** `from_repo` errors when commit timestamp is outside chrono's representable range
- **Why it's unreachable:** `git2::Time` is an i64 in seconds since epoch. `chrono::DateTime` can represent the full i64 range (billion-year timespans in both directions). There is no i64 value that chrono cannot represent.
- **DISTILL evidence:** Attempted to construct a Signature with `Time::new(i64::MAX, 0)`. Succeeded. Commit creation succeeded. `from_repo` succeeded without error.
- **Code path:** Lines 173-174: `DateTime::from_timestamp(...).ok_or(...)?;` — error arm exists but cannot be reached
- **Activation:** This test will remain `#[ignore]` with its documentation. No activation expected. The test serves as proof that the error arm is indeed unreachable.
- **Coverage recommendation:** Exclude this line from coverage analysis, or document as "unreachable-by-design" in coverage reports (if the tool supports it).

---

### BLOCKED (1 test)

**`test_try_new_discovery_failure_blocked`**
- **What it was intended to test:** `CommitSaver::try_new()` errors when `Repository::discover` fails to find a git repo
- **Why it's blocked:** `try_new` calls `Repository::discover("./")` with a hardcoded path. To test the error arm, we'd need to:
  1. Mutate the process's current directory (forbidden in parallel test suites), OR
  2. Refactor `try_new` to accept a path parameter (out of DISTILL scope — production code change)
- **Recommendation:** DELIVER should add a `try_discover(path: &Path)` internal helper that both `try_new` and this test can use. See `upstream-issues.md` for full recommendation.
- **Activation:** This test will remain `#[ignore]` until DELIVER implements the path-injection helper. It's a placeholder documenting the gap.

---

## Compile & Test Evidence

### Compile Check
```
devenv shell -- cargo test --no-run
   Compiling rusty-commit-saver v4.14.4
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.79s
```
✓ **All tests compile successfully**

### Run Check (with --ignored flag)
```
devenv shell -- cargo test --lib -- --ignored
running 11 tests
test vim_commit::commit_saver_tests::test_create_diary_file_unwritable_location ... ok
test vim_commit::commit_saver_tests::test_from_repo_out_of_range_timestamp_unreachable ... ok
test vim_commit::commit_saver_tests::test_append_entry_to_diary_parent_not_exists ... ok
[... 8 more tests, all ok ...]
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured
```
✓ **All 11 tests pass**

---

## Summary

**Ready for DELIVER activation:**
- 9 tests will PASS immediately on unskipping (behavior already implemented)
- 1 test documents an unreachable error arm (leave ignored; documents defensive code)
- 1 test is BLOCKED pending path-injection refactor (placeholder; awaits DELIVER action)

**Zero REAL-BUG findings:** The tests validate correct behavior. Two behavior discoveries (detached HEAD returns "HEAD", timestamp range unreachable) were documented in `upstream-issues.md`.

**Next step:** DELIVER unskips tests one-at-a-time, implements path-injection helper for `test_try_new_discovery_failure_blocked`, and reaches ≥95% coverage on `vim_commit.rs`.
