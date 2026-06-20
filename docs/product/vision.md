# Product Vision — rusty-commit-saver

A small Rust CLI, run as a Git hook, that records each commit's metadata
(folder, time, message, repository URL, branch, hash) as a Markdown table row in
a dated Obsidian diary entry. It turns an otherwise invisible commit history into
a searchable, tag-organized daily journal inside the user's Obsidian vault.

## Who it serves

Developers who keep an Obsidian vault and want an automatic, zero-effort log of
what they worked on, when, and where — without manually maintaining a worklog.

## Product shape

- Entry point: a Git hook invoking the `rusty-commit-saver` binary.
- Configuration: INI file at `~/.config/rusty-commit-saver/rusty-commit-saver.ini`.
- Output: dated diary files with YAML frontmatter, Obsidian tags, and a commit
  table the tool appends to.

## SSOT note

This file and `jobs.yaml` were bootstrapped during the DISCUSS wave for feature
`vim-commit-coverage` (an infrastructure / quality feature). They are intentionally
minimal — kept proportionate to a small CLI. Expand them when a user-facing
feature warrants real JTBD analysis.
