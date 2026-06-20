# Slice 02 — CommitSaver construction error branches `@infrastructure`

> Elephant-carpaccio slice for feature `vim-commit-coverage`. Maps to **US-02**.
> Escape-valve note: only `@infrastructure` stories — exempt via the infrastructure
> escape valve (Decision 1/4), not a violation of the slice-composition gate.

## Goal

Cover the `# Errors` arms of `CommitSaver::from_repo` / `try_new` / `default` with
fault-injection tests built on `tempfile` repositories.

## IN scope

- Test: no-HEAD repo → `head()?` error arm returns `Err` (no panic).
- Test: detached HEAD → `no_branch_set` placeholder branch.
- Test: out-of-range commit timestamp → `ok_or` error arm.
- Test: `try_new` discovery failure (injected path, never `set_current_dir`).

## OUT of scope

- Filesystem IO error arms (Slice 03) and path-inspection arms (Slice 04).
- Any production-code change.

## Learning hypothesis

We believe building repos via `from_repo(&repo)` + `tempfile` exercises every
construction error arm without process-global state — confirming the constructors
fail gracefully (Err, not panic) on broken repositories.

## Acceptance criteria

- [ ] `from_repo` no-HEAD test asserts `Err`, covers `head()?` arm.
- [ ] `from_repo` detached-HEAD test covers `no_branch_set` branch.
- [ ] Test covers the out-of-range-timestamp `ok_or` arm (or documents it unreachable).
- [ ] `try_new` test covers `Repository::discover` failure without mutating CWD.
- [ ] Suite green in parallel under `devenv shell -- cargo test`.

## Dependencies

**Slice 01** (single instrumented copy) — so the covered arms count once.

## Effort

≤ 1 day.

## Reference class

git2 fault-injection unit tests; well-trodden except the out-of-range timestamp,
which may require a crafted `git2::Time` — tracked risk, may end documented-unreachable.

## Carpaccio taste tests

- **End-to-end?** Yes — `test-coverage` reflects the newly-green arms.
- **Demonstrable?** Yes — red→green lines on the construction functions.
- **Thin?** Yes — one cohesive group of error arms.
- **Independent value?** Yes — proves graceful failure on malformed repos.
