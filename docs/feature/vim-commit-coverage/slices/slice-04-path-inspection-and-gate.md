# Slice 04 — Path-inspection error branches + cross the 95% gate `@infrastructure`

> Elephant-carpaccio slice for feature `vim-commit-coverage`. Maps to **US-04**.
> Escape-valve note: only `@infrastructure` stories — exempt via the infrastructure
> escape valve (Decision 1/4), not a violation of the slice-composition gate.

## Goal

Cover the boundary `# Errors` arms of `check_diary_path_exists` (not found) and
`get_parent_from_full_path` (root / no parent), library-side, and confirm
`vim_commit.rs` clears the codecov 95% project gate.

## IN scope

- Test: `check_diary_path_exists` on a missing path → `"Path does not exist!"` error.
- Test: `get_parent_from_full_path(Path::new("/"))` → no-parent error.
- Happy-path counterparts (existing file → Ok; nested path → parent resolves).
- Ensure `main_tests` equivalents exist library-side so they count once.
- Final: confirm `vim_commit.rs` ≥ 95% under `devenv shell -- test-coverage`.

## OUT of scope

- Construction arms (Slice 02) and IO arms (Slice 03).
- Any production-code change.

## Learning hypothesis

We believe these two boundary arms are the last genuine gaps; covering them
library-side pushes `vim_commit.rs` over 95% and flips the codecov project status
from failing to passing.

## Acceptance criteria

- [ ] Test covers `check_diary_path_exists` not-found error arm.
- [ ] Test covers `get_parent_from_full_path` root/no-parent error arm.
- [ ] Happy-path counterparts confirm the success arms.
- [ ] `vim_commit.rs` line coverage ≥ 95% under `devenv shell -- test-coverage`.
- [ ] codecov **project** gate (target 95%) passes on the PR.

## Dependencies

**Slices 01, 02, 03** — this is the closing slice that crosses the gate.

## Effort

≤ 1 day.

## Reference class

Boundary-condition unit tests + a final gate verification; lowest-risk slice,
mostly relocating already-existing assertions to the library copy.

## Carpaccio taste tests

- **End-to-end?** Yes — ends at the codecov gate status, the feature's true outcome.
- **Demonstrable?** Yes — gate flips failing→passing; coverage % crosses 95%.
- **Thin?** Yes — two boundary arms + final verification.
- **Independent value?** Yes — delivers the headline outcome (passing 95% gate).
