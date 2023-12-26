use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use git2::Repository;
use log::info;
use std::fs::OpenOptions;
use std::io::Write;
// use std::iter::Zip;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CommitSaver {
    pub repository_url: String,
    pub commit_branch_name: String,
    pub commit_hash: String,
    pub commit_msg: String,
    pub commit_datetime: DateTime<Utc>,
}

/// Defaults for CommitSaver
impl Default for CommitSaver {
    fn default() -> CommitSaver {
        let git_repo = Repository::discover("./").unwrap();
        let head = git_repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        CommitSaver {
            repository_url: {
                let bind = git_repo.find_remote("origin").unwrap();
                bind.url().unwrap().replace('\"', "")
            },
            commit_branch_name: { head.shorthand().unwrap().replace('\"', "") },
            commit_hash: { commit.id().to_string() },
            commit_msg: {
                let commit = commit.message().unwrap().replace(['\n', '\"'], "");

                match commit.char_indices().nth(120) {
                    None => commit.to_string(),
                    Some((idx, _)) => commit[..idx].to_string(),
                }
            },
            commit_datetime: {
                let commit_date: i64 = commit.time().seconds();
                DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(commit_date, 0).unwrap(),
                    Utc,
                )
            },
        }
    }
}

impl CommitSaver {
    pub fn new() -> Self {
        CommitSaver::default()
    }

    /// Prepares input to write to vimwiki
    /// TODO datetime fix
    fn prepare_commit_entry_as_string(&mut self) -> String {
        let _dt = self.commit_datetime.format("%H:%M:%S");
        "| {_dt:} | {self.commit_msg:} | {self.repository_url:} | {self.commit_branch_name:} | {self.commit_hash:} |\n".to_string()
    }

    /// Append git stuff to diary
    pub fn append_entry_to_diary(&mut self, wiki: &PathBuf) {
        let new_commit_str = self.prepare_commit_entry_as_string();

        let mut file_ref = OpenOptions::new()
            .append(true)
            .open(wiki)
            .expect("[ERROR] Unable to open wiki");

        file_ref
            .write_all(new_commit_str.as_bytes())
            .expect("[ERROR] Failed to write the new commit string");

        info!(
            "[INFO] Commit logged in ...................................................... {:}",
            wiki.file_name()
                .expect("[ERROR] No filename found while writing commit to the file")
                .to_str()
                .unwrap()
        );
    }

    pub fn prepare_path_for_commit(&mut self) -> String {
        let diary_path = prepare_path_with_emojis();
        let paths_with_dates_and_file = self.prepare_date_for_commit_file();
        format!(".vimwiki/{diary_path:}/0. Commits/{paths_with_dates_and_file:}")
    }

    pub fn prepare_date_for_commit_file(&mut self) -> String {
        // %B	July	Full month name. Also accepts corresponding abbreviation in parsing.
        // %F	2001-07-08	Year-month-day format (ISO 8601). Same as %Y-%m-%d.
        self.commit_datetime.format("%Y/%m-%B/%F.md").to_string()
    }
}

fn prepare_path_with_emojis() -> String {
    let calendar = emojis::get("📅").unwrap();
    let diary = format!("{calendar:} Diaries");
    diary
}
