#!/usr/bin/env bash
# The gate the Rust tests cannot be: a REAL git commit through a REAL
# post-commit hook running this binary.
#
# The failures this repo has actually shipped were config faults at the hook
# boundary, where the binary runs with no RUST_LOG and nobody reads the log -
# so the thing worth asserting is what a human sees on stderr, and whether the
# diary entry appeared. Everything runs in a throwaway repo with a throwaway
# vault and an explicit core.hooksPath, so a machine-wide hooks directory
# cannot interfere and nothing touches the real vault.
#
# Usage:
#   tests/hook-gate.sh [config-kind] [path-to-binary]
#
# config-kind: good | unknown-key | blank-key | missing-key | unknown-section
#              (default: good)
# binary:      default target/debug/rusty-commit-saver
#
# Prints the git commit exit status, the hook's stderr, and what was
# journalled. Read it - the point is the human-visible output, so this
# reports rather than asserts.
set -uo pipefail

kind=${1:-good}
bin=${2:-target/debug/rusty-commit-saver}

[ -x "$bin" ] || {
  echo "hook-gate: no binary at $bin - run: cargo build" >&2
  exit 2
}
bin=$(realpath "$bin")

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

vault="$work/vault"
cfg="$work/rusty-commit-saver.ini"
hooks="$work/hooks"
repo="$work/repo"
mkdir -p "$vault" "$hooks" "$repo"

{
  echo '[obsidian]'
  echo "root_path_dir = $vault"
  case "$kind" in
  missing-key) : ;;                  # commit_path absent entirely
  blank-key) echo 'commit_path =' ;; # present but empty
  *) echo 'commit_path = Diaries/Commits' ;;
  esac
  echo
  echo '[templates]'
  echo 'commit_date_path = %Y/%m-%B/%F.md'
  echo 'commit_datetime = %H:%M:%S'
  case "$kind" in
  unknown-key) echo 'commit_datetimes = %H:%M' ;;
  esac
  echo
  echo '[exclude]'
  echo 'repos = some-other-repo'
  case "$kind" in
  unknown-section)
    echo
    echo '[future_release]'
    echo 'key = value'
    ;;
  esac
} >"$cfg"

printf '#!/bin/sh\nexec "%s"\n' "$bin" >"$hooks/post-commit"
chmod +x "$hooks/post-commit"

git init -q "$repo"
git -C "$repo" config user.email gate@example.invalid
git -C "$repo" config user.name 'hook gate'
git -C "$repo" config core.hooksPath "$hooks"
echo hello >"$repo/file.txt"
git -C "$repo" add file.txt

RUSTY_COMMIT_SAVER_CONFIG="$cfg" \
  git -C "$repo" commit -q -m 'test: drive the real post-commit hook' \
  >"$work/stdout" 2>"$work/stderr"
rc=$?

echo "config kind    : $kind"
echo "binary         : $bin"
echo "git commit exit: $rc"
echo "commit created : $(git -C "$repo" rev-parse --short HEAD 2>/dev/null || echo NONE)"
echo '--- hook stderr ---'
cat "$work/stderr"
echo '--- journalled ---'
if find "$vault" -type f | grep -q .; then
  find "$vault" -type f -printf '%P\n'
else
  echo '(nothing written)'
fi
