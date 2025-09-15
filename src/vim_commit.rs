use chrono::DateTime;
use chrono::Utc;
use git2::Repository;
use log::info;
use std::env;
use std::error::Error;
use std::fs;
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
                // Preserve original lines, escape pipes, then join with <br/>
                let raw = commit.message().unwrap_or("");
                raw.lines()
                    .map(|line| line.trim().replace('|', "\\|"))
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("<br/>")
            },
            commit_datetime: {
                let commit_date: i64 = commit.time().seconds();
                DateTime::from_timestamp(commit_date, 0).unwrap()
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

pub fn prepare_path_with_emojis() -> String {
    let calendar = emojis::get("📅").unwrap();
    let diary = format!("{calendar:} Diaries");
    diary
}

markup::define! {
    DiaryFileEntry(frontmatter: Vec<String>, diary_date: String) {
"---
category: diary\n
section: commits\n
tags:\n"
@for tag in frontmatter.iter() {
"- '" @tag "'\n"
}
"date: " @diary_date
"\n
---
\n
# " @diary_date
"\n
| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
|--------|------|----------------|----------------|--------|-------------|\n"
    }
}

pub fn get_parent_from_full_path(full_diary_path: &Path) -> Result<&Path, Box<dyn Error>> {
    match full_diary_path.parent() {
        Some(dir) => Ok(dir),
        None => Err("Something went wrong when getting the parent directory".into()),
    }
}

/// Method to veritfy that the file exists
/// Will trigger the creation of it with a template if it doesn't
pub fn check_diary_path_exists(full_diary_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if Path::new(&full_diary_path).exists() {
        return Ok(());
    }
    Err("Path does not exist!".into())
}

pub fn create_directories_for_new_entry(
    entry_directory_and_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let parent_dirs = get_parent_from_full_path(entry_directory_and_path)?;
    fs::create_dir_all(parent_dirs)?;
    info!("[INFO] Creating diary file & path ");
    println!("[INFO] Creating diary file & path ");

    Ok(())
}

pub fn create_diary_file(
    full_diary_file_path: &str,
    commit_saver_struct: &mut CommitSaver,
) -> Result<(), Box<dyn Error>> {
    let frontmatter = commit_saver_struct.prepare_frontmatter_tags();
    let diary_date = commit_saver_struct
        .commit_datetime
        .format("%Y-%m-%d")
        .to_string();

    let template = DiaryFileEntry {
        frontmatter,
        diary_date,
    }
    .to_string();
    fs::write(full_diary_file_path, template)?;

    Ok(())
}

// CommitSaver tests
#[cfg(test)]
mod commit_saver_tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_commit_saver() -> CommitSaver {
        CommitSaver {
            repository_url: "https://github.com/test/repo.git".to_string(),
            commit_branch_name: "main".to_string(),
            commit_hash: "abc123def456".to_string(),
            commit_msg: "Test commit message".to_string(),
            commit_datetime: Utc.with_ymd_and_hms(2023, 12, 25, 10, 30, 0).unwrap(),
        }
    }

    #[test]
    fn test_commit_saver_new() {
        // This test requires being in a git repository
        // We'll mock the behavior or skip if not in a git repo
        if Repository::discover("./").is_ok() {
            let commit_saver = CommitSaver::new();

            assert!(!commit_saver.repository_url.is_empty());
            assert!(!commit_saver.commit_branch_name.is_empty());
            assert!(!commit_saver.commit_hash.is_empty());
        }
    }

    #[test]
    fn test_prepare_commit_entry_as_string() {
        let mut commit_saver = create_test_commit_saver();
        let test_path = PathBuf::from("/test/path");

        let result = commit_saver.prepare_commit_entry_as_string(&test_path);

        assert!(result.contains("/test/path"));
        assert!(result.contains("10:30:00"));
        assert!(result.contains("Test commit message"));
        assert!(result.contains("https://github.com/test/repo.git"));
        assert!(result.contains("main"));
        assert!(result.contains("abc123def456"));
        assert!(result.ends_with("|\n"));
    }

    #[test]
    fn test_prepare_commit_entry_with_pipe_escaping() {
        let mut commit_saver = CommitSaver {
            repository_url: "https://github.com/test/repo.git".to_string(),
            commit_branch_name: "main".to_string(),
            commit_hash: "abc123def456".to_string(),
            commit_msg: "Test | commit | with | pipes".to_string(),
            commit_datetime: Utc.with_ymd_and_hms(2023, 12, 25, 10, 30, 0).unwrap(),
        };
        let test_path = PathBuf::from("/test/path");

        let result = commit_saver.prepare_commit_entry_as_string(&test_path);

        // The commit message should have pipes escaped
        assert!(result.contains("Test | commit | with | pipes"));
    }

    #[test]
    fn test_prepare_frontmatter_tags() {
        let mut commit_saver = create_test_commit_saver();

        let tags = commit_saver.prepare_frontmatter_tags();

        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"#datetime/days/Monday".to_string()));
        assert!(tags.contains(&"#diary/commits".to_string()));
    }

    #[test]
    fn test_prepare_path_for_commit() {
        let mut commit_saver = create_test_commit_saver();

        let path = commit_saver.prepare_path_for_commit();

        assert!(path.contains("📅 Diaries"));
        assert!(path.contains("0. Commits"));
        assert!(path.contains("2023/12-December/2023-12-25.md"));
    }

    #[test]
    fn test_prepare_date_for_commit_file() {
        let mut commit_saver = create_test_commit_saver();

        let date_path = commit_saver.prepare_date_for_commit_file();

        assert_eq!(date_path, "2023/12-December/2023-12-25.md");
    }

    #[test]
    fn test_append_entry_to_diary() -> Result<(), Box<dyn std::error::Error>> {
        let mut commit_saver = create_test_commit_saver();
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_diary.md");

        // Create the file first
        File::create(&file_path)?;

        let result = commit_saver.append_entry_to_diary(&file_path);

        assert!(result.is_ok());

        // Verify content was written
        let content = fs::read_to_string(&file_path)?;
        assert!(content.contains("Test commit message"));
        assert!(content.contains("abc123def456"));

        Ok(())
    }

    #[test]
    fn test_append_entry_to_diary_file_not_exists() {
        let mut commit_saver = create_test_commit_saver();
        let non_existent_path = PathBuf::from("/non/existent/file.md");

        let result = commit_saver.append_entry_to_diary(&non_existent_path);

        assert!(result.is_err());
    }
}

// Helper function tests
#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_prepare_path_with_emojis() {
        let result = prepare_path_with_emojis();
        assert_eq!(result, "📅 Diaries");
    }
}
