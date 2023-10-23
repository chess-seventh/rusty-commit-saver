use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use git2::Repository;
use log::info;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CommitSaver {
    pub repository_url: String,
    pub branch_name: String,
    pub commit_hash: String,
    pub commit_msg: String,
    pub datetime: String,
}

/// Defaults for CommitSaver
impl Default for CommitSaver {
    fn default() -> CommitSaver {
        CommitSaver {
            repository_url: {
                let git_repo = Repository::discover("./").unwrap();
                let bind = git_repo.find_remote("origin").unwrap();
                bind.url().unwrap().replace('\"', "")
            },
            branch_name: {
                let git_repo = Repository::discover("./").unwrap();
                let head = git_repo.head().unwrap();
                head.shorthand().unwrap().replace('\"', "")
            },
            commit_hash: {
                let git_repo = Repository::discover("./").unwrap();
                let head = git_repo.head().unwrap();
                let cbind = head.peel_to_commit().unwrap();
                cbind.id().to_string()
            },
            commit_msg: {
                let git_repo = Repository::discover("./").unwrap();
                let head = git_repo.head().unwrap();
                let cbind = head.peel_to_commit().unwrap();
                let commit = cbind.message().unwrap().replace(['\n', '\"'], "");

                match commit.char_indices().nth(120) {
                    None => commit.to_string(),
                    Some((idx, _)) => commit[..idx].to_string(),
                }
            },
            datetime: {
                let git_repo = Repository::discover("./").unwrap();
                let head = git_repo.head().unwrap();

                let cbind = head.peel_to_commit().unwrap();
                let commit_date: i64 = cbind.time().seconds();
                let dt: DateTime<Utc> = DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(commit_date, 0).unwrap(),
                    Utc,
                );
                dt.format("%H:%M:%S").to_string()
            },
        }
    }
}

impl CommitSaver {
    pub fn new() -> Self {
        CommitSaver::default()
    }

    /// Prepares input to write to vimwiki
    fn prepare_input(&mut self) -> String {
        format!(
            "| {:} | {:} | {:} | {:} | {:} |\n",
            self.datetime, self.commit_msg, self.repository_url, self.branch_name, self.commit_hash
        )
    }

    /// Append git stuff to diary
    pub fn append_commit_stuff_to_diary(&mut self, wiki: &PathBuf) {
        let new_commit_str = self.prepare_input();

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

    /// check if remote is for transics or else.
    pub fn select_proper_diary_folder(&mut self) -> String {
        let diary_path = prepare_path_with_emojis();
        if self.repository_url.contains("transics") {
            return format!(".vimwiki/{diary_path:}/3. Field Work");
        }
        let diary_path = format!("{diary_path:}/0. Daily");
        format!(".vimwiki/{diary_path:}/")
    }
}

fn prepare_path_with_emojis() -> String {
    let calendar = emojis::get("📅").unwrap();
    let diary = format!("{calendar:} Diaries");
    diary
}
