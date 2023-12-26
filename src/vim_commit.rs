use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use git2::Repository;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
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
    fn prepare_commit_entry_as_string(&mut self, path: &Path) -> String {
        format!(
            "| {:} | {:} | {:} | {:} | {:} | {:} |\n",
            path.display(),
            self.commit_datetime.format("%H:%M:%S"),
            self.commit_msg,
            self.repository_url,
            self.commit_branch_name,
            self.commit_hash
        )
    }

    pub fn prepare_frontmatter_tags(&mut self) -> Vec<String> {
        let week_number = format!("#datetime/week/{:}", self.commit_datetime.format("%W"));
        let week_day = format!("#datetime/days/{:}", self.commit_datetime.format("%A"));

        vec![week_number, week_day, "#diary/commits".to_string()]
    }

    pub fn prepare_path_for_commit(&mut self) -> String {
        let diary_path = prepare_path_with_emojis();
        let paths_with_dates_and_file = self.prepare_date_for_commit_file();
        format!("{diary_path:}/0. Commits/{paths_with_dates_and_file:}")
    }

    fn prepare_date_for_commit_file(&mut self) -> String {
        // %B	July	Full month name. Also accepts corresponding abbreviation in parsing.
        // %F	2001-07-08	Year-month-day format (ISO 8601). Same as %Y-%m-%d.
        self.commit_datetime.format("%Y/%m-%B/%F.md").to_string()
    }

    /// Append commit to existing diary
    pub fn append_entry_to_diary(&mut self, wiki: &PathBuf) -> Result<(), Box<dyn Error>> {
        let path = env::current_dir()?;
        let new_commit_str = self.prepare_commit_entry_as_string(&path);

        println!("{new_commit_str:}");
        println!("{:}", wiki.display());
        let mut file_ref = OpenOptions::new().append(true).open(wiki)?;

        file_ref.write_all(new_commit_str.as_bytes())?;

        Ok(())
    }
}

fn prepare_path_with_emojis() -> String {
    let calendar = emojis::get("📅").unwrap();
    let diary = format!("{calendar:} Diaries");
    diary
}
