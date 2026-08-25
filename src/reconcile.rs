//! The `git log` backstop for the commit journal (D-98).
//!
//! The post-commit hook only writes the journal on a box that mounts the
//! vault. Everywhere else the row is lost, and once the config was copied onto
//! those boxes it started being lost *silently* — which is worse, because a
//! journal that quietly stops looks exactly like a quiet week.
//!
//! This module is the other half of the ruling: a pass that reads a
//! repository's history and appends whatever row the day note is missing. It
//! needs no broker and no network, so it works on the day the wire is down and
//! on the day the wire does not exist yet.
//!
//! # What it will not do
//!
//! It **only ever appends**. It never rewrites, reorders or reformats a row
//! that is already in a note. The day notes are hand-readable files a human
//! keeps, so a backfill that touched existing rows would be a far worse defect
//! than a missing one.
//!
//! Identity is the **commit hash column**, which is why a row written by the
//! hook and a row this pass would have written collapse to one: whichever
//! arrives first wins, and the other is recognised as already present. That is
//! the dedup rule D-98 named as the thing that must be exactly right.

use chrono::DateTime;
use chrono::Utc;
use git2::Repository;
use git2::Sort;

use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use log::info;
use log::warn;

use crate::vim_commit::CommitSaver;
use crate::vim_commit::canonical_repo_name;
use crate::vim_commit::ensure_diary_file;
use crate::vim_commit::is_repo_excluded;

/// What one reconcile pass over one repository did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReconcileReport {
    /// The repository's canonical name, as the exclude list spells it.
    pub repository: String,

    /// The repository is in the configured exclude list, so nothing was read
    /// and nothing was written.
    pub excluded: bool,

    /// Commits walked. Zero on an excluded repository.
    pub scanned: usize,

    /// Rows appended to a day note by this pass.
    pub appended: usize,

    /// Commits whose row was already in its day note — the hook got there
    /// first, or an earlier pass did.
    pub already_present: usize,
}

/// Every commit hash already recorded as a row in a day note.
///
/// The hash is the row's last column, and a commit id contains no `|`, so the
/// text after the final separator is read directly. Anything that is not
/// plausibly an object id — the table header, the `|---|` rule, a stray line —
/// is dropped, which is what keeps `COMMIT HASH` out of the set.
#[must_use]
pub fn hashes_in_note(contents: &str) -> HashSet<String> {
    contents
        .lines()
        .filter_map(last_table_column)
        .filter(|value| looks_like_object_id(value))
        .map(str::to_owned)
        .collect()
}

/// The final cell of a Markdown table row, or `None` if the line is not one.
///
/// Splitting from the right matters: a commit message may carry escaped pipes
/// in an earlier column, and reading from the left would have to understand
/// that escaping. The last column never can, so it does not have to.
fn last_table_column(line: &str) -> Option<&str> {
    let line = line.trim();
    let inner = line.strip_prefix('|')?.strip_suffix('|')?;

    inner.rsplit('|').next().map(str::trim)
}

/// Whether a cell could be a git object id.
///
/// Deliberately permissive about length so an abbreviated hash still matches —
/// this decides what is *already recorded*, and a false negative would append
/// a duplicate row, which is the one outcome that matters.
fn looks_like_object_id(value: &str) -> bool {
    (7..=40).contains(&value.len()) && value.chars().all(|character| character.is_ascii_hexdigit())
}

/// Appends every row a repository's day notes are missing.
///
/// Walks the history reachable from `HEAD`, oldest commit first, and for each
/// one appends a row to the note for **that commit's own date** unless the note
/// already carries its hash.
///
/// # Arguments
///
/// * `repo` - the repository to reconcile
/// * `obsidian_root_path_dir` - the vault root
/// * `obsidian_commit_path` - the commits subdirectory under the root
/// * `template_commit_date_path` - chrono format for the note's path
/// * `template_commit_datetime` - chrono format for the row's TIME column
/// * `excluded_repos` - repositories to skip entirely
/// * `since` - when set, commits older than this are not walked
///
/// # Errors
///
/// Returns an error if the history cannot be walked, if a note cannot be
/// created, or if a note cannot be read or appended to.
pub fn reconcile_repo(
    repo: &Repository,
    obsidian_root_path_dir: &Path,
    obsidian_commit_path: &Path,
    template_commit_date_path: &str,
    template_commit_datetime: &str,
    excluded_repos: &[String],
    since: Option<DateTime<Utc>>,
) -> Result<ReconcileReport, Box<dyn Error>> {
    let repository = canonical_repo_name(repo).unwrap_or_else(|| "unknown".to_string());

    let mut report = ReconcileReport {
        repository: repository.clone(),
        ..ReconcileReport::default()
    };

    if is_repo_excluded(&repository, excluded_repos) {
        info!("[reconcile_repo()]: repo '{repository}' is excluded; reading nothing.");
        report.excluded = true;
        return Ok(report);
    }

    let head = repo.head()?;
    let branch = head.shorthand().unwrap_or("no_branch_set").to_string();

    // The hook reports the directory it ran in. This pass runs from wherever
    // its timer put it, so the honest FOLDER is the repository's own work tree.
    let folder = repo
        .workdir()
        .map_or_else(|| repo.path().to_path_buf(), Path::to_path_buf);

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME | Sort::REVERSE)?;
    revwalk.push_head()?;

    // One entry per day note touched, so a note is read once however many of
    // its rows this pass appends.
    let mut known: HashMap<PathBuf, HashSet<String>> = HashMap::new();

    for oid in revwalk {
        let commit = repo.find_commit(oid?)?;
        let mut saver = CommitSaver::from_commit(repo, &commit, &branch)?;

        if let Some(floor) = since {
            if saver.commit_datetime < floor {
                continue;
            }
        }

        report.scanned += 1;

        let note = saver.diary_path_for(
            obsidian_root_path_dir,
            obsidian_commit_path,
            template_commit_date_path,
        );

        if !known.contains_key(&note) {
            let recorded = if note.exists() {
                hashes_in_note(&fs::read_to_string(&note)?)
            } else {
                HashSet::new()
            };
            known.insert(note.clone(), recorded);
        }

        let recorded = known
            .get_mut(&note)
            .expect("the note's row set was just inserted");

        if recorded.contains(&saver.commit_hash) {
            report.already_present += 1;
            continue;
        }

        ensure_diary_file(&note, &mut saver)?;
        saver.append_row_to_diary(&note, &folder, template_commit_datetime)?;

        recorded.insert(saver.commit_hash.clone());
        report.appended += 1;
    }

    info!(
        "[reconcile_repo()]: '{repository}': {} scanned, {} appended, {} already present.",
        report.scanned, report.appended, report.already_present
    );

    Ok(report)
}

/// Reconciles every repository in `repo_paths`, in order.
///
/// A repository that cannot be opened or walked is reported on stderr and the
/// pass continues to the next one. A reconcile that abandoned the remaining
/// repositories because one of them is broken would leave the journal in a
/// worse state than not running at all, and the next run would hit the same
/// repository and stop in the same place.
///
/// # Errors
///
/// Never returns an error for a single bad repository; the `Err` arm is
/// reserved for a caller-level fault.
pub fn reconcile_all(
    repo_paths: &[PathBuf],
    obsidian_root_path_dir: &Path,
    obsidian_commit_path: &Path,
    template_commit_date_path: &str,
    template_commit_datetime: &str,
    excluded_repos: &[String],
    since: Option<DateTime<Utc>>,
) -> Result<Vec<ReconcileReport>, Box<dyn Error>> {
    let mut reports = Vec::new();

    for repo_path in repo_paths {
        let outcome = match Repository::discover(repo_path) {
            Ok(repo) => reconcile_repo(
                &repo,
                obsidian_root_path_dir,
                obsidian_commit_path,
                template_commit_date_path,
                template_commit_datetime,
                excluded_repos,
                since,
            ),
            Err(error) => Err(error.into()),
        };

        match outcome {
            Ok(report) => reports.push(report),
            Err(error) => {
                warn!(
                    "[reconcile_all()]: skipping {}: {error}",
                    repo_path.display()
                );
                // Also on stderr: the reconciler runs from a timer, where
                // env_logger caps the level at Error without RUST_LOG and the
                // warning would be swallowed.
                eprintln!(
                    "rusty-commit-saver: skipping {}: {error}",
                    repo_path.display()
                );
            }
        }
    }

    Ok(reports)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod reconcile_tests {
    use super::*;
    use chrono::TimeZone;
    use git2::Signature;
    use git2::Time;
    use tempfile::TempDir;

    const DATE_TEMPLATE: &str = "%Y/%m-%B/%F.md";
    const TIME_TEMPLATE: &str = "%H:%M:%S";

    /// A repository whose commits sit at chosen instants, so a test can assert
    /// which day note a row lands in without depending on the clock.
    struct Fixture {
        _dir: TempDir,
        repo: Repository,
    }

    impl Fixture {
        fn new(origin: &str) -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let repo = Repository::init(dir.path()).expect("init");
            repo.remote("origin", origin).expect("remote");

            Fixture { _dir: dir, repo }
        }

        /// Commits an empty tree at `epoch_seconds`, returning the new id.
        fn commit_at(&self, message: &str, epoch_seconds: i64) -> String {
            let when = Time::new(epoch_seconds, 0);
            let who = Signature::new("Test User", "test@example.com", &when).expect("signature");

            let tree_id = self
                .repo
                .index()
                .expect("index")
                .write_tree()
                .expect("tree");
            let tree = self.repo.find_tree(tree_id).expect("find tree");

            let parents = match self.repo.head().ok().and_then(|h| h.peel_to_commit().ok()) {
                Some(parent) => vec![parent],
                None => Vec::new(),
            };
            let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();

            self.repo
                .commit(Some("HEAD"), &who, &who, message, &tree, &parent_refs)
                .expect("commit")
                .to_string()
        }
    }

    fn vault() -> TempDir {
        tempfile::tempdir().expect("vault tempdir")
    }

    fn run(
        fixture: &Fixture,
        root: &Path,
        excluded: &[String],
        since: Option<DateTime<Utc>>,
    ) -> ReconcileReport {
        reconcile_repo(
            &fixture.repo,
            root,
            Path::new("Diaries/Commits"),
            DATE_TEMPLATE,
            TIME_TEMPLATE,
            excluded,
            since,
        )
        .expect("reconcile should succeed")
    }

    fn note_for(root: &Path, year: i32, month: u32, day: u32) -> PathBuf {
        let stamp = Utc
            .with_ymd_and_hms(year, month, day, 0, 0, 0)
            .single()
            .expect("a real date");

        root.join("Diaries")
            .join("Commits")
            .join(stamp.format(DATE_TEMPLATE).to_string())
    }

    // 2024-03-05 09:00:00 UTC and 2024-04-11 18:30:00 UTC.
    const MARCH_FIFTH: i64 = 1_709_629_200;
    const APRIL_ELEVENTH: i64 = 1_712_860_200;

    #[test]
    fn hashes_in_note_reads_the_last_column_only() {
        let note = "\
| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
|--------|------|----------------|----------------|--------|-------------|
| /src/x | 09:00:00 | feat: a thing | https://h/x.git | main | abc123def456 |
| /src/x | 10:00:00 | fix: another | https://h/x.git | main | 0123456789abcdef |
";

        let hashes = hashes_in_note(note);

        assert_eq!(
            hashes.len(),
            2,
            "both rows, and neither header line: {hashes:?}"
        );
        assert!(hashes.contains("abc123def456"));
        assert!(hashes.contains("0123456789abcdef"));
    }

    #[test]
    fn hashes_in_note_survives_an_escaped_pipe_in_the_message() {
        // The writer escapes pipes in the message column. Reading the row from
        // the left would have to understand that; reading the last cell does
        // not, and this is the row shape that proves it.
        let note =
            "| /src/x | 09:00:00 | fix: a \\| b \\| c | https://h/x.git | main | deadbeef1234 |\n";

        let hashes = hashes_in_note(note);

        assert_eq!(hashes.len(), 1, "the escaped pipes must not confuse it");
        assert!(hashes.contains("deadbeef1234"));
    }

    #[test]
    fn hashes_in_note_ignores_prose_and_frontmatter() {
        let note = "---\ncategory: diary\n---\n\n# 2024-03-05\n\nsome prose\n";

        assert!(hashes_in_note(note).is_empty());
    }

    #[test]
    fn a_repo_with_no_note_gets_every_row() {
        let fixture = Fixture::new("git@github.com:chess-seventh/example.git");
        fixture.commit_at("feat: one", MARCH_FIFTH);
        fixture.commit_at("feat: two", MARCH_FIFTH + 60);
        let root = vault();

        let report = run(&fixture, root.path(), &[], None);

        assert_eq!(report.scanned, 2);
        assert_eq!(report.appended, 2);
        assert_eq!(report.already_present, 0);

        let note = fs::read_to_string(note_for(root.path(), 2024, 3, 5)).expect("note written");
        assert!(note.contains("feat: one"), "first row missing: {note}");
        assert!(note.contains("feat: two"), "second row missing: {note}");
    }

    #[test]
    fn a_second_pass_appends_nothing() {
        // MUST-PROVE: running the reconciler twice produces exactly one row per
        // commit. This is the dedup rule with the wire taken out of it — the
        // same identity check that makes the hook and this pass collapse to one
        // row when both run.
        let fixture = Fixture::new("git@github.com:chess-seventh/example.git");
        fixture.commit_at("feat: one", MARCH_FIFTH);
        fixture.commit_at("feat: two", MARCH_FIFTH + 60);
        let root = vault();

        run(&fixture, root.path(), &[], None);
        let after = fs::read_to_string(note_for(root.path(), 2024, 3, 5)).expect("note");

        let second = run(&fixture, root.path(), &[], None);

        assert_eq!(second.appended, 0, "a second pass must append nothing");
        assert_eq!(second.already_present, 2);
        assert_eq!(
            fs::read_to_string(note_for(root.path(), 2024, 3, 5)).expect("note"),
            after,
            "the note must be byte-identical after a second pass"
        );
    }

    #[test]
    fn a_row_the_hook_already_wrote_is_not_written_again() {
        // The dedup rule from the other direction: the row is in the note but
        // this pass never wrote it, exactly as it would be on the vault box
        // where the hook runs too.
        let fixture = Fixture::new("git@github.com:chess-seventh/example.git");
        let sha = fixture.commit_at("feat: one", MARCH_FIFTH);
        let root = vault();

        let note = note_for(root.path(), 2024, 3, 5);
        fs::create_dir_all(note.parent().expect("parent")).expect("dirs");
        fs::write(
            &note,
            format!("| /elsewhere | 09:00:00 | feat: one | u | main | {sha} |\n"),
        )
        .expect("seed the note");

        let report = run(&fixture, root.path(), &[], None);

        assert_eq!(report.appended, 0, "the hook's row already covers it");
        assert_eq!(report.already_present, 1);
        assert_eq!(
            fs::read_to_string(&note).expect("note").lines().count(),
            1,
            "the note must still hold exactly one row"
        );
    }

    #[test]
    fn existing_content_is_never_rewritten() {
        // MUST-PROVE: the pass only ever APPENDS. The day notes are Franci's,
        // and a backfill that reformatted or reordered what is already there
        // would be a far worse defect than a missing row.
        let fixture = Fixture::new("git@github.com:chess-seventh/example.git");
        fixture.commit_at("feat: one", MARCH_FIFTH);
        let root = vault();

        let note = note_for(root.path(), 2024, 3, 5);
        fs::create_dir_all(note.parent().expect("parent")).expect("dirs");
        let hand_written =
            "# a note I wrote by hand\n\n| /x | 00:00:00 | older | u | main | 1111111 |\n";
        fs::write(&note, hand_written).expect("seed");

        run(&fixture, root.path(), &[], None);

        let after = fs::read_to_string(&note).expect("note");
        assert!(
            after.starts_with(hand_written),
            "existing bytes must survive untouched: {after}"
        );
        assert!(after.contains("feat: one"), "the new row must be appended");
    }

    #[test]
    fn a_row_lands_in_the_note_for_its_own_date() {
        // MUST-PROVE: never today's note. Without this the first backfill
        // collapses a month of history into whatever note the clock names.
        let fixture = Fixture::new("git@github.com:chess-seventh/example.git");
        fixture.commit_at("feat: march", MARCH_FIFTH);
        fixture.commit_at("feat: april", APRIL_ELEVENTH);
        let root = vault();

        run(&fixture, root.path(), &[], None);

        let march = fs::read_to_string(note_for(root.path(), 2024, 3, 5)).expect("march note");
        let april = fs::read_to_string(note_for(root.path(), 2024, 4, 11)).expect("april note");

        assert!(march.contains("feat: march"));
        assert!(!march.contains("feat: april"), "april leaked into march");
        assert!(april.contains("feat: april"));
        assert!(!april.contains("feat: march"), "march leaked into april");

        let today = note_for(
            root.path(),
            Utc::now().format("%Y").to_string().parse().expect("year"),
            Utc::now().format("%m").to_string().parse().expect("month"),
            Utc::now().format("%d").to_string().parse().expect("day"),
        );
        assert!(
            !today.exists(),
            "nothing may be written to today's note: {}",
            today.display()
        );
    }

    #[test]
    fn an_excluded_repo_is_never_read_or_written() {
        // MUST-PROVE: the exclusion was enforced only on the hook path. A
        // reconciler that ignored it would journal every claude-src baton, which
        // is precisely what the exclude list exists to prevent.
        let fixture = Fixture::new("git@github.com:chess-seventh/claude-src.git");
        fixture.commit_at("chore(mailbox): a baton", MARCH_FIFTH);
        let root = vault();

        let report = run(&fixture, root.path(), &["claude-src".to_string()], None);

        assert!(report.excluded, "the repo must be reported as excluded");
        assert_eq!(report.scanned, 0, "an excluded repo is not even walked");
        assert_eq!(report.appended, 0);
        assert!(
            !root.path().join("Diaries").exists(),
            "an excluded repo must create no vault output at all"
        );
    }

    #[test]
    fn the_exclude_list_matches_the_origin_not_the_directory() {
        // The worktree a lane builds in is named after the lane, so matching on
        // the directory would let every excluded repo back in through its own
        // worktrees.
        let fixture = Fixture::new("git@github.com:chess-seventh/claude-src.git");
        fixture.commit_at("chore(mailbox): a baton", MARCH_FIFTH);
        let root = vault();

        let report = run(&fixture, root.path(), &["claude-src".to_string()], None);

        assert_eq!(report.repository, "claude-src");
        assert!(report.excluded);
    }

    #[test]
    fn since_leaves_older_commits_alone() {
        let fixture = Fixture::new("git@github.com:chess-seventh/example.git");
        fixture.commit_at("feat: march", MARCH_FIFTH);
        fixture.commit_at("feat: april", APRIL_ELEVENTH);
        let root = vault();

        let floor = DateTime::from_timestamp(APRIL_ELEVENTH - 3600, 0).expect("a real instant");
        let report = run(&fixture, root.path(), &[], Some(floor));

        assert_eq!(report.scanned, 1, "only the april commit is in range");
        assert_eq!(report.appended, 1);
        assert!(
            !note_for(root.path(), 2024, 3, 5).exists(),
            "a commit below the floor must not create its note"
        );
    }

    #[test]
    fn reconcile_all_reports_a_bad_path_and_carries_on() {
        let fixture = Fixture::new("git@github.com:chess-seventh/example.git");
        fixture.commit_at("feat: one", MARCH_FIFTH);
        let root = vault();
        let not_a_repo = tempfile::tempdir().expect("tempdir");

        let reports = reconcile_all(
            &[
                not_a_repo.path().to_path_buf(),
                fixture.repo.workdir().expect("workdir").to_path_buf(),
            ],
            root.path(),
            Path::new("Diaries/Commits"),
            DATE_TEMPLATE,
            TIME_TEMPLATE,
            &[],
            None,
        )
        .expect("reconcile_all should not fail on one bad path");

        assert_eq!(reports.len(), 1, "the good repository is still reconciled");
        assert_eq!(reports[0].appended, 1);
    }
}
