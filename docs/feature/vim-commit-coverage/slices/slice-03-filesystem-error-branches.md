# Slice 03 — Diary filesystem error branches `@infrastructure`

> Elephant-carpaccio slice for feature `vim-commit-coverage`. Maps to **US-03**.
> Escape-valve note: only `@infrastructure` stories — exempt via the infrastructure
> escape valve (Decision 1/4), not a violation of the slice-composition gate.

## Goal

Cover the IO `# Errors` arms of `append_entry_to_diary`, `create_diary_file`, and
`create_directories_for_new_entry`, against the single library copy.

## IN scope

- Test: append to a non-existent diary file → `open(wiki)?` error arm.
- Test: create a diary file in a read-only/forbidden location → `fs::write?` arm.
- Test: create directories under a forbidden path → `fs::create_dir_all?` arm.
- Relocate/duplicate relevant `main_tests` cases into the library test module so
  they count toward the single instrumented copy.

## OUT of scope

- Construction arms (Slice 02) and path-inspection arms (Slice 04).
- Any production-code change.

## Learning hypothesis

We believe `tempfile` paths that are missing or read-only exercise every IO
failure arm — confirming the diary writers surface `Err` (logged by `main`)
rather than losing data silently.

## Acceptance criteria

- [ ] Test covers `append_entry_to_diary` open-failure arm.
- [ ] Test covers `create_diary_file` write-failure arm.
- [ ] Test covers `create_directories_for_new_entry` mkdir-failure arm.
- [ ] All three count against the library copy (post Slice 01), not the binary.
- [ ] Suite green in parallel under `devenv shell -- cargo test`.

## Dependencies

**Slice 01** (single instrumented copy). Independent of Slices 02 and 04.

## Effort

≤ 1 day.

## Reference class

Filesystem fault-injection tests (read-only dirs, `/proc`-style paths) on Linux;
devenv is Linux, so platform-specific assertions are acceptable.

## Carpaccio taste tests

- **End-to-end?** Yes — `test-coverage` reflects the newly-green IO arms.
- **Demonstrable?** Yes — red→green on the diary-writing functions.
- **Thin?** Yes — one cohesive group of IO error arms.
- **Independent value?** Yes — proves safe failure on disk/permission errors.
