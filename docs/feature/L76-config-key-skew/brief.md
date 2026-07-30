# Bug-fix brief - L76 config key skew

> Lean nWave motion, standard-rigor gate. Second axis of the outage L66 closed:
> 4.17.3 made an unrecognised config SECTION non-fatal, but the KEYS were left
> as they were. Root cause known up front; this brief records the defect, the
> decision, the gate, and the acceptance scenarios that drive
> RED -> GREEN -> COMMIT.

## Defect

One INI file at `~/.config/rusty-commit-saver/rusty-commit-saver.ini` is shared
by every checkout on the machine, while the binary reading it is pinned
per-repo and per-home-manager-generation. Config and binary therefore drift
apart by design, and the config parser has two axes of skew: sections and keys.
Only sections were made tolerant.

Measured on 4.17.3, by driving real commits through a real post-commit hook
(`git init` repo, `core.hooksPath` pointing at a hook that execs the binary):

| config fault | what the user sees | journalled |
|---|---|---|
| unrecognised section | one stderr line naming the section | yes |
| unrecognised key | **nothing at all** | yes |
| missing required key | `thread 'main' panicked at src/config.rs:862:14: Could not get commit_path from config` + backtrace note | no |

Both key rows are wrong, and they compound:

1. **An unrecognised key is swallowed in total silence.** `configparser` simply
   never returns a key nobody asks for, and there is no key-level equivalent of
   `KNOWN_SECTIONS` (`src/config.rs:617`). A `commit_datetimes` typo, or a key
   renamed by a newer release, applies nothing and says nothing.
2. **A required key that is absent panics with a message that does not name the
   config file.** Four sites hard-`.expect()` on the `Option`:
   `src/config.rs:738-747` (`commit_datetime`), `819-830` (`commit_date_path`),
   `859-885` (`commit_path`), `920-950` (`root_path_dir`). The panic text names
   the key but not the file, and `retrieve_config_file_path()`
   (`src/config.rs:1050`) returns the file *contents*, so the resolved path is
   not retained anywhere for an error message to use.

Together they make a rename the same outage as L66 through a different door:
the new key is ignored silently (1), the old key is missing, so the run panics
(2) - and the message points at a key rather than at the file to edit.

## Decision: fatal, but say why

A post-commit hook cannot abort a commit - git ignores its exit status, which
the gate below verifies (`git commit` exits 0 and the commit exists in every
fault case). So the real cost of a config fault is stderr noise plus a
silently missing diary entry, never a lost commit.

Given that, the policy is:

- **Unrecognised key -> warn and continue.** Same reasoning as the section fix:
  a config written for a newer release must not brick an older binary.
- **Missing required key -> still fatal**, because without it there is no
  destination to write to. Degrading to "journal nothing, exit 0" would make a
  broken config indistinguishable from a quiet day, and the diary would stop
  for weeks unnoticed. The fix is the message, not the severity: name the
  resolved config file and the exact `[section] key`.
- Both warnings go to **stderr as well as the log**, because the hook runs
  without `RUST_LOG` and `env_logger` caps the level at Error there - a
  log-only warning is invisible in practice.

Decided by Franci, 2026-07-30, over "never fatal, always degrade" and "fatal
only when there is no destination".

Two findings from the root-cause pass widened that decision, both reproduced
before acting on them:

- **A blank value defeats a presence check.** `commit_path =` satisfied
  `Option::is_some`, exited 0, and journalled into the vault *root* instead of
  the configured folder - a wrong-location write with no error at all. An
  empty or whitespace value therefore counts as missing. A test that blessed
  the same hole for `root_path_dir` (empty vault root resolving to `/`) is
  withdrawn deliberately rather than worked around.
- **`commit_datetime` was required and never read.** The TIME column was
  hardcoded to `%H:%M:%S`, so a typo in that key could abort a run over a
  value nothing consumed - fatal-and-ignored, which no policy can defend.
  Franci's call: wire it up, so the key means what it says. The live config
  already carries `%H:%M:%S`, so no diary output changes.

## Fix

- New `KNOWN_KEYS` table beside `KNOWN_SECTIONS` (`src/config.rs`), listing what
  each known section understands.
- New `unrecognised_keys()` returning the sorted `[section] key` list, and
  `report_unrecognised_keys()` warning it to log + stderr, called from
  `set_obsidian_vars()`.
- `GlobalVars` gains `config_path`, set in `set_all()` from the resolved path,
  so an error can name the file. `get_ini_file_at()` / `read_config_file()`
  split out for that; `get_ini_file()` and `retrieve_config_file_path()` keep
  their signatures and behaviour.
- New `require_key()` replaces the four `.expect()` calls with one fatal path
  whose message names the file, the key, and the unrecognised keys of that same
  section; it rejects a blank value as missing.
- `[templates] commit_datetime` is threaded from `main()` through
  `run_commit_saver()` and `append_entry_to_diary()` to the row builder in
  `src/vim_commit.rs`, which had the format hardcoded.

## Gate (acceptance scenarios)

1. A key the binary does not know, in a section it does -> named on stderr,
   run continues, commit still journalled. *(the silent half)*
2. A required key absent -> run aborts with a message naming the resolved
   config file and `[section] key`, no bare "Could not get X from config".
   *(the unhelpful half)*
3. An unrecognised **section** -> still warns and continues (4.17.3 behaviour
   must not regress).
4. A good config -> no warning at all on stderr, entry journalled as before.
5. A required key present but **blank** -> treated as missing, same message,
   nothing journalled. Previously: exit 0 and a diary in the wrong directory.
6. The configured time format reaches the diary row, instead of the hardcoded
   one.

Verified by unit tests in `src/config.rs`, plus - and this is the point of the
lane - the real-hook gate: a genuine `git commit` in a throwaway repo whose
`core.hooksPath` runs the built binary against each config above, asserting the
stderr text, the commit's exit status, and whether a diary file appeared.
Script: `tests/hook-gate.sh`. Full gate: `devenv shell -- pre-check`.

## Deploy (Franci)

Merge to master -> the release workflow bumps + tags from the conventional
`fix:` commit; then `up-hm` deploys the new binary. The machine's global
post-commit hook still pins 4.17.0, which is unaffected by the current config
(it knows `[obsidian]`, `[templates]` and `[exclude]`) - verified, no live
outage waiting on this deploy.
