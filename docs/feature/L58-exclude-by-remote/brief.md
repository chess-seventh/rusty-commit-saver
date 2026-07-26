# Bug-fix brief — L58 exclude-by-remote

> Lean nWave motion (rigor: lean). Standalone defect in delivered behaviour,
> found while cleaning up hook artifacts in L57. Root cause already known;
> this brief records the defect, the gate, and the acceptance scenarios that
> drive RED → GREEN → COMMIT.

## Defect

`[exclude]` in the runtime ini fails to skip a repository when the commit is
made from a **git worktree**. `run_commit_saver` identified the repo via
`current_repo_workdir_name()` — `repo.workdir()` → `Path::file_name()`, i.e. the
**worktree directory basename** — and `is_repo_excluded()` compares it by exact
`==`. A worktree's basename is the lane name (e.g. `l58-wt`), not the repo name,
so `[exclude] repos = claude-src` never matches and every worktree commit floods
the Obsidian diary. This blocks clean use of the L49 `wt` worktree workflow.

## Fix

Resolve **canonical repo identity** from the `origin` remote URL (already read as
`repository_url` in `CommitSaver::from_repo`), which is stable across all
worktrees of a repo. Fall back to the workdir basename when there is no usable
`origin` (local-only repos). One `claude-src` exclude entry then covers every
worktree.

- New (in `src/vim_commit.rs`): `repo_name_from_url`, `canonical_repo_name(&Repository)`,
  `current_repo_canonical_name`.
- Changed: `run_commit_saver` (`src/main.rs`) calls `current_repo_canonical_name`;
  `is_repo_excluded` doc updated (it now receives the canonical name).
- The 4 pure `is_repo_excluded` string-predicate tests are unchanged.

## Gate (acceptance scenarios)

1. A commit made in a **worktree** whose `origin` is `…/claude-src.git` (basename
   ≠ `claude-src`) → **excluded**, writes no diary row. *(the regression)*
2. A commit in an **excluded repo** in its normal checkout → still excluded,
   writes nothing.
3. A commit in a **non-excluded repo** → still journals normally.

Verified by: `test_canonical_repo_name_prefers_origin_over_workdir` (1),
`test_run_commit_saver_skips_excluded_repo` updated to the canonical name (2),
plus `repo_name_from_url` variant/sentinel tests. Full gate:
`devenv shell -- pre-check` (clippy `-D warnings` + tests + build).

## Deploy (Franci)

Merge to master → the `🎯 Release` workflow bumps + tags from the conventional
`fix:` commit; then `up-hm` deploys the new binary.
