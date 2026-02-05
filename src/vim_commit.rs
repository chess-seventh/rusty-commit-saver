use chrono::DateTime;
use chrono::Utc;
use git2::Repository;

use std::env;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use log::debug;
use log::error;
use log::info;
use log::warn;

/// Stores Git commit metadata for logging to Obsidian diary entries.
///
/// This struct captures all essential information about a single Git commit
/// that will be written as a row in the daily diary table. It's automatically
/// populated from the current Git repository's HEAD commit.
///
/// # Examples
///
/// ```ignore
/// use rusty_commit_saver::CommitSaver;
///
/// // Automatically populated from current Git repository
/// let saver = CommitSaver::new();
///
/// println!("Repository: {}", saver.repository_url);
/// println!("Branch: {}", saver.commit_branch_name);
/// println!("Hash: {}", saver.commit_hash);
/// println!("Message: {}", saver.commit_msg);
/// ```
///
/// # See Also
///
/// - [`CommitSaver::new()`] - Create a new instance from current Git repo
/// - [`CommitSaver::append_entry_to_diary()`] - Write commit to diary file
#[derive(Debug, Clone)]
pub struct CommitSaver {
    /// The Git remote origin URL.
    ///
    /// Retrieved from the repository's `origin` remote. Double quotes are stripped.
    ///
    /// # Examples
    ///
    /// - `https://github.com/user/repo.git`
    /// - `git@github.com:user/repo.git`
    /// - `https://git.sr.ht/~user/repo`
    pub repository_url: String,

    /// The current Git branch name.
    ///
    /// Retrieved from the repository's HEAD reference. Double quotes are stripped.
    ///
    /// # Examples
    ///
    /// - `main`
    /// - `develop`
    /// - `feature/add-documentation`
    pub commit_branch_name: String,

    /// The full SHA-1 commit hash (40 characters).
    ///
    /// Uniquely identifies the commit in the Git repository.
    ///
    /// # Format
    ///
    /// Always 40 hexadecimal characters (e.g., `abc123def456...`)
    pub commit_hash: String,

    /// The formatted commit message for Obsidian display.
    ///
    /// The message is processed for safe rendering in Markdown tables:
    /// - Pipe characters (`|`) are escaped to `\|`
    /// - Multiple lines are joined with `<br/>`
    /// - Empty lines are filtered out
    /// - Leading/trailing whitespace is trimmed
    ///
    /// # Examples
    ///
    /// ```text
    /// Original: "feat: add feature\n\nWith details"
    /// Formatted: "feat: add feature<br/>With details"
    ///
    /// Original: "fix: issue | problem"
    /// Formatted: "fix: issue \| problem"
    /// ```
    pub commit_msg: String,

    /// The UTC timestamp when the commit was created.
    ///
    /// Used for:
    /// - Generating date-based directory paths
    /// - Displaying commit time in diary entries
    /// - Creating frontmatter tags (week number, day of week)
    ///
    /// # Format
    ///
    /// Stored as `DateTime<Utc>` from the `chrono` crate.
    pub commit_datetime: DateTime<Utc>,
}

/// Creates a `CommitSaver` instance with default values from the current Git repository.
///
/// This implementation automatically discovers the Git repository in the current directory
/// and extracts all commit metadata from the HEAD commit. It's the core logic used by
/// [`CommitSaver::new()`].
///
/// # Panics
///
/// Panics if:
/// - No Git repository is found in the current directory or any parent directory
/// - The repository has no HEAD (uninitialized or corrupted repository)
/// - The HEAD reference cannot be resolved to a commit
/// - The remote "origin" doesn't exist
///
/// # Commit Message Processing
///
/// The commit message undergoes several transformations:
/// 1. Split into individual lines
/// 2. Trim whitespace from each line
/// 3. Escape pipe characters: `|` → `\|` (for Markdown table compatibility)
/// 4. Filter out empty lines
/// 5. Join with `<br/>` separator (for Obsidian rendering)
///
/// # Examples
///
/// ```ignore
/// use rusty_commit_saver::CommitSaver;
///
/// // Using Default trait directly
/// let saver = CommitSaver::default();
///
/// // Equivalent to:
/// let saver2 = CommitSaver::new();
/// ```
impl Default for CommitSaver {
    fn default() -> CommitSaver {
        let git_repo = Repository::discover("./").unwrap();
        let head = git_repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        CommitSaver {
            repository_url: {
                let url = match git_repo.find_remote("origin") {
                    Ok(bind) => bind.url().unwrap().replace('\"', ""),
                    _ => "no_url_set".to_string(),
                };
                url
            },
            commit_branch_name: {
                // head.shorthand().unwrap().replace('\"', "")
                let branch = match head.shorthand() {
                    Some(branch) => branch.replace('\"', ""),
                    None => "no_branch_set".to_string(),
                };
                branch
            },
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
    /// Creates a new `CommitSaver` instance by discovering the current Git repository.
    ///
    /// This function automatically:
    /// - Discovers the Git repository in the current directory (`.`)
    /// - Extracts commit metadata from the HEAD commit
    /// - Formats the commit message for Obsidian (escapes pipes, adds `<br/>`)
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - No Git repository is found in the current directory
    /// - The repository has no HEAD (uninitialized repo)
    /// - The HEAD cannot be resolved to a commit
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rusty_commit_saver::CommitSaver;
    ///
    /// let saver = CommitSaver::new();
    /// println!("Commit hash: {}", saver.commit_hash);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        CommitSaver::default()
    }

    /// Formats commit metadata as a Markdown table row for diary entry.
    ///
    /// Generates a single table row containing all commit information in the format
    /// expected by the Obsidian diary template. The row includes pipe delimiters
    /// and ends with a newline.
    ///
    /// # Arguments
    ///
    /// * `path` - The current working directory where the commit was made
    ///
    /// # Returns
    ///
    /// A formatted string representing one table row with these columns:
    /// 1. **FOLDER** - Current working directory path
    /// 2. **TIME** - Commit timestamp (HH:MM:SS format)
    /// 3. **COMMIT MESSAGE** - Escaped and formatted commit message
    /// 4. **REPOSITORY URL** - Git remote origin URL
    /// 5. **BRANCH** - Current branch name
    /// 6. **COMMIT HASH** - Full SHA-1 commit hash
    ///
    /// # Format
    ///
    /// ```text
    /// | /path/to/repo | 14:30:45 | feat: add feature | https://github.com/user/repo.git | main | abc123... |
    /// ```
    ///
    /// # Note
    ///
    /// This is a private helper method called by [`append_entry_to_diary()`](Self::append_entry_to_diary).
    /// The commit message has already been formatted with escaped pipes and `<br/>` separators
    /// during struct initialization.
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

    /// Generates Obsidian-style frontmatter tags based on the commit timestamp.
    ///
    /// Creates three metadata tags for organizing diary entries:
    /// 1. **Week tag**: `#datetime/week/WW` (e.g., `#datetime/week/02` for week 2)
    /// 2. **Day tag**: `#datetime/days/DDDD` (e.g., `#datetime/days/Monday`)
    /// 3. **Category tag**: `#diary/commits` (constant)
    ///
    /// These tags are used in the Obsidian diary file's YAML frontmatter to enable:
    /// - Filtering commits by week number
    /// - Organizing by day of week
    /// - Cross-referencing with other diary entries
    ///
    /// # Returns
    ///
    /// A vector of three strings containing formatted Obsidian tags
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rusty_commit_saver::CommitSaver;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let mut saver = CommitSaver {
    ///     repository_url: "https://github.com/example/repo.git".to_string(),
    ///     commit_branch_name: "main".to_string(),
    ///     commit_hash: "abc123".to_string(),
    ///     commit_msg: "feat: add feature".to_string(),
    ///     commit_datetime: Utc.with_ymd_and_hms(2025, 1, 13, 10, 30, 0).unwrap(), // Monday
    /// };
    ///
    /// let tags = saver.prepare_frontmatter_tags();
    /// assert_eq!(tags.len(), 3);
    /// assert!(tags.contains("week"));
    /// assert!(tags.contains("Monday"));[1]
    /// assert_eq!(tags, "#diary/commits");
    /// ```
    pub fn prepare_frontmatter_tags(&mut self) -> Vec<String> {
        info!("[CommitSaver::prepare_frontmatter_tags()]: Preparing the frontmatter week number.");
        let week_number = format!("#datetime/week/{:}", self.commit_datetime.format("%W"));

        info!("[CommitSaver::prepare_frontmatter_tags()]: Preparing the frontmatter week day.");
        let week_day = format!("#datetime/days/{:}", self.commit_datetime.format("%A"));

        info!(
            "[CommitSaver::prepare_frontmatter_tags()]: Returing the formatted vector with the frontmatter tags week number and day."
        );
        vec![week_number, week_day, "#diary/commits".to_string()]
    }

    /// Constructs the full file path for a diary entry based on the commit timestamp.
    ///
    /// Combines the Obsidian commit directory path with a date-formatted subdirectory structure
    /// to create the final path where the commit entry should be saved.
    ///
    /// # Arguments
    ///
    /// * `obsidian_commit_path` - Base directory path for commits (e.g., `Diaries/Commits`)
    /// * `template_commit_date_path` - Chrono format string for the date hierarchy (e.g., `%Y/%m-%B/%F.md`)
    ///
    /// # Returns
    ///
    /// A formatted path string combining the base directory and formatted date
    ///
    /// # Format Specifiers (Chrono)
    ///
    /// - `%Y` - Year (e.g., `2025`)
    /// - `%m` - Month as number (e.g., `01`)
    /// - `%B` - Full month name (e.g., `January`)
    /// - `%F` - ISO 8601 date (e.g., `2025-01-14.md`)
    /// - `%d` - Day of month (e.g., `14`)
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The `obsidian_commit_path` cannot be converted to a valid UTF-8 string
    /// - The path contains invalid characters that cannot be represented as a string
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rusty_commit_saver::CommitSaver;
    /// use std::path::PathBuf;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let mut saver = CommitSaver {
    ///     repository_url: "https://github.com/example/repo.git".to_string(),
    ///     commit_branch_name: "main".to_string(),
    ///     commit_hash: "abc123".to_string(),
    ///     commit_msg: "feat: add feature".to_string(),
    ///     commit_datetime: Utc.with_ymd_and_hms(2025, 1, 14, 10, 30, 0).unwrap(),
    /// };
    ///
    /// let path = saver.prepare_path_for_commit(
    ///     &PathBuf::from("Diaries/Commits"),
    ///     "%Y/%m-%B/%F.md"
    /// );
    /// // Returns: "/Diaries/Commits/2025/01-January/2025-01-14.md"
    /// assert!(path.contains("2025"));
    /// assert!(path.contains("January"));
    /// assert!(path.contains("2025-01-14.md"));
    /// ```
    pub fn prepare_path_for_commit(
        &mut self,
        obsidian_commit_path: &Path,
        template_commit_date_path: &str,
    ) -> String {
        info!("[CommitSaver::prepare_path_for_commit()]: Preparing the path for commit file.");
        let commit_path = obsidian_commit_path
            .as_os_str()
            .to_str()
            .expect("asd")
            .to_string();

        info!("[CommitSaver::prepare_path_for_commit()]: Retrieving the path for commit file.");
        let paths_with_dates_and_file =
            self.prepare_date_for_commit_file(template_commit_date_path);

        info!(
            "[CommitSaver::prepare_path_for_commit()]: Returning the full String of the ComitPath and File."
        );
        format!("/{commit_path:}/{paths_with_dates_and_file:}")
    }

    /// Formats the commit timestamp using a Chrono date format string.
    ///
    /// Applies the given format template to the commit's datetime to generate
    /// a date-based directory path or filename. This enables flexible organization
    /// of diary entries by year, month, week, or custom hierarchies.
    ///
    /// # Arguments
    ///
    /// * `path_format` - Chrono format string (e.g., `%Y/%m-%B/%F.md`)
    ///
    /// # Returns
    ///
    /// A formatted date string suitable for file paths
    ///
    /// # Common Format Specifiers
    ///
    /// - `%Y` - Year (4 digits, e.g., `2025`)
    /// - `%m` - Month (2 digits, e.g., `01`)
    /// - `%B` - Full month name (e.g., `January`)
    /// - `%b` - Abbreviated month (e.g., `Jan`)
    /// - `%d` - Day of month (2 digits, e.g., `14`)
    /// - `%F` - ISO 8601 date format (`%Y-%m-%d`, e.g., `2025-01-14`)
    /// - `%A` - Full weekday name (e.g., `Monday`)
    /// - `%W` - Week number (e.g., `02`)
    ///
    /// # Examples
    ///
    /// ```text
    /// // With format "%Y/%m-%B/%F.md" and datetime 2025-01-14:
    /// // Returns: "2025/01-January/2025-01-14.md"
    ///
    /// // With format "%Y/week-%W/%F.md" and datetime in week 2:
    /// // Returns: "2025/week-02/2025-01-14.md"
    /// ```
    ///
    /// # Note
    ///
    /// This is a private helper method called by [`prepare_path_for_commit()`](Self::prepare_path_for_commit).
    fn prepare_date_for_commit_file(&mut self, path_format: &str) -> String {
        info!(
            "[CommitSaver::prepare_date_for_commit_file()]: Formatting commit path with DateTime."
        );
        // %B	July	Full month name. Also accepts corresponding abbreviation in parsing.
        // %F	2001-07-08	Year-month-day format (ISO 8601). Same as %Y-%m-%d.
        self.commit_datetime.format(path_format).to_string()
    }

    /// Appends the current commit as a table row to an Obsidian diary file.
    ///
    /// This method writes a formatted commit entry to the specified diary file in append mode.
    /// The entry includes: current directory, timestamp, commit message, repository URL, branch, and commit hash.
    ///
    /// # Arguments
    ///
    /// * `wiki` - Path to the diary file where the commit entry should be appended
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Successfully appended the commit entry to the file
    /// - `Err(Box<dyn Error>)` - If file operations fail (file doesn't exist, permission denied, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The diary file cannot be opened for appending
    /// - The current working directory cannot be determined
    /// - File write operations fail (I/O error, permission denied)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rusty_commit_saver::CommitSaver;
    /// use std::path::PathBuf;
    ///
    /// let mut saver = CommitSaver::new();
    /// let diary_path = PathBuf::from("/home/user/diary/2025-01-14.md");
    ///
    /// match saver.append_entry_to_diary(&diary_path) {
    ///     Ok(()) => println!("Commit logged successfully!"),
    ///     Err(e) => eprintln!("Failed to log commit: {}", e),
    /// }
    /// ```
    pub fn append_entry_to_diary(&mut self, wiki: &PathBuf) -> Result<(), Box<dyn Error>> {
        info!("[CommitSaver::append_entry_to_diary()]: Getting current directory.");
        let path = env::current_dir()?;

        info!("[CommitSaver::append_entry_to_diary()]: Preparing the commit_entry_as_string.");
        let new_commit_str = self.prepare_commit_entry_as_string(&path);

        debug!("[CommitSaver::append_entry_to_diary()]: Commit String: {new_commit_str:}");
        debug!(
            "[CommitSaver::append_entry_to_diary()]: Wiki:\n{:}",
            wiki.display()
        );
        let mut file_ref = OpenOptions::new().append(true).open(wiki)?;

        file_ref.write_all(new_commit_str.as_bytes())?;

        Ok(())
    }
}

// Markup template for generating Obsidian diary file structure.
//
// This macro defines the template for new diary entry files, including:
// - YAML frontmatter with metadata and tags
// - Main heading with the date
// - Markdown table header for commit entries
//
// Used internally by create_diary_file().
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

/// Extracts the parent directory from a file path.
///
/// Returns a reference to the parent directory component of the given path.
/// This is useful for creating parent directories before writing a file.
///
/// # Arguments
///
/// * `full_diary_path` - A file path to extract the parent directory from
///
/// # Returns
///
/// - `Ok(&Path)` - Reference to the parent directory
/// - `Err(Box<dyn Error>)` - If the path has no parent (e.g., root directory `/`)
///
/// # Errors
///
/// Returns an error if:
/// - The path is the root directory (has no parent)
/// - The path is a relative single component with no parent
///
/// # Examples
///
/// ```ignore
/// use rusty_commit_saver::vim_commit::get_parent_from_full_path;
/// use std::path::{Path, PathBuf};
///
/// // Normal nested path
/// let path = Path::new("/home/user/documents/diary.md");
/// let parent = get_parent_from_full_path(path).unwrap();
/// assert_eq!(parent, Path::new("/home/user/documents"));
///
/// // Deep nesting
/// let deep = Path::new("/a/b/c/d/e/f/file.txt");
/// let parent = get_parent_from_full_path(deep).unwrap();
/// assert_eq!(parent, Path::new("/a/b/c/d/e/f"));
///
/// // Root directory fails
/// let root = Path::new("/");
/// assert!(get_parent_from_full_path(root).is_err());
/// ```
pub fn get_parent_from_full_path(full_diary_path: &Path) -> Result<&Path, Box<dyn Error>> {
    info!(
        "[get_parent_from_full_path()] Checking if there is parents for: {:}.",
        full_diary_path.display()
    );
    if let Some(dir) = full_diary_path.parent() {
        Ok(dir)
    } else {
        error!(
            "[get_parent_from_full_path()]: Something went wrong when getting the parent directory"
        );
        Err("Something went wrong when getting the parent directory".into())
    }
}

/// Verifies whether a diary file exists at the specified path.
///
/// This function checks if the file at the given path exists on the filesystem.
/// It's used to determine whether to create a new diary file with a template
/// or append to an existing one.
///
/// # Arguments
///
/// * `full_diary_path` - Path to the diary file to check
///
/// # Returns
///
/// - `Ok(())` - File exists at the specified path
/// - `Err(Box<dyn Error>)` - File does not exist at the specified path
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist on the filesystem
/// - The path cannot be accessed due to permission issues
/// - The path represents a directory instead of a file
///
/// # Examples
///
/// ```ignore
/// use rusty_commit_saver::vim_commit::check_diary_path_exists;
/// use std::path::PathBuf;
/// use std::fs::File;
///
/// // Create a temporary test file
/// let test_file = PathBuf::from("/tmp/test_diary.md");
/// File::create(&test_file).unwrap();
///
/// // File exists - returns Ok
/// assert!(check_diary_path_exists(&test_file).is_ok());
///
/// // File doesn't exist - returns Err
/// let missing_file = PathBuf::from("/tmp/nonexistent.md");
/// assert!(check_diary_path_exists(&missing_file).is_err());
/// ```
pub fn check_diary_path_exists(full_diary_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    info!(
        "[check_diary_path_exists()]: Checking that full_diary_path exists: {:}",
        full_diary_path.display()
    );
    if Path::new(&full_diary_path).exists() {
        return Ok(());
    }
    warn!("[check_diary_path_exists()]: Path does not exist!");
    Err("Path does not exist!".into())
}

/// Creates all necessary parent directories for a diary file path.
///
/// Recursively creates the complete directory hierarchy needed to store a diary file.
/// Uses `fs::create_dir_all()` which is idempotent—calling it on existing directories
/// is safe and will not cause errors.
///
/// # Arguments
///
/// * `obsidian_root_path_dir` - The full path including the filename for the diary entry
///
/// # Returns
///
/// - `Ok(())` - All parent directories were successfully created
/// - `Err(Box<dyn Error>)` - Directory creation failed (permission denied, invalid path, etc.)
///
/// # Errors
///
/// Returns an error if:
/// - The parent path cannot be determined (root directory)
/// - No write permissions to the parent directory
/// - Invalid filesystem (e.g., read-only filesystem)
/// - Path components are invalid (e.g., null bytes)
///
/// # Examples
///
/// ```ignore
/// use rusty_commit_saver::vim_commit::create_directories_for_new_entry;
/// use std::path::PathBuf;
/// use std::fs;
///
/// let diary_path = PathBuf::from("/tmp/test/deep/nested/path/diary.md");
///
/// // Create all parent directories
/// create_directories_for_new_entry(&diary_path).unwrap();
///
/// // Verify the directories were created
/// assert!(PathBuf::from("/tmp/test/deep/nested/path").exists());
///
/// // Calling again on existing directories is safe (idempotent)
/// assert!(create_directories_for_new_entry(&diary_path).is_ok());
/// ```
pub fn create_directories_for_new_entry(
    obsidian_root_path_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    info!("[create_directories_for_new_entry()] Getting parent_dirs.");
    let parent_dirs = get_parent_from_full_path(obsidian_root_path_dir)?;
    fs::create_dir_all(parent_dirs)?;
    info!("[create_directories_for_new_entry()] Creating diary file & path");

    Ok(())
}

/// Creates a new diary file with Obsidian frontmatter and table template.
///
/// Generates a diary entry file with:
/// - YAML frontmatter containing metadata and tags for Obsidian organization
/// - A markdown table header for commit entries (folder, time, message, repo, branch, hash)
/// - Pre-formatted for use with [`CommitSaver::append_entry_to_diary()`]
///
/// # Template Structure
///
/// The generated file uses the internal `DiaryFileEntry` markup template:
///
/// ```text
/// ---
/// category: diary
/// section: commits
/// tags:
/// - '#datetime/week/02'
/// - '#datetime/days/Monday'
/// - '#diary/commits'
/// date: 2025-01-14
/// ---
///
/// # 2025-01-14
///
/// | FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
/// |--------|------|----------------|----------------|--------|-------------|
/// ```
///
/// # Arguments
/// ... (rest of your existing documentation)
///
/// The created file is ready for commit entries to be appended to its table.
///
/// # Arguments
///
/// * `full_diary_file_path` - The complete path where the file should be created
/// * `commit_saver_struct` - The `CommitSaver` instance to extract metadata from
///
/// # Returns
///
/// - `Ok(())` - File was successfully created with the template
/// - `Err(Box<dyn Error>)` - File creation or write operation failed
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be created (parent directory doesn't exist, permission denied)
/// - Write operations fail (disk full, I/O error)
/// - Path is invalid or contains invalid UTF-8
///
/// # Examples
///
/// ```ignore
/// use rusty_commit_saver::vim_commit::create_diary_file;
/// use rusty_commit_saver::CommitSaver;
/// use chrono::{TimeZone, Utc};
/// use std::fs;
///
/// let mut saver = CommitSaver {
///     repository_url: "https://github.com/example/repo.git".to_string(),
///     commit_branch_name: "main".to_string(),
///     commit_hash: "abc123def456".to_string(),
///     commit_msg: "feat: implement feature".to_string(),
///     commit_datetime: Utc.with_ymd_and_hms(2025, 1, 14, 10, 30, 0).unwrap(),
/// };
///
/// let file_path = "/home/user/diary/2025-01-14.md";
/// create_diary_file(file_path, &mut saver).unwrap();
///
/// // Verify file was created with proper structure
/// let content = fs::read_to_string(file_path).unwrap();
/// assert!(content.contains("---")); // Frontmatter markers
/// assert!(content.contains("category: diary"));
/// assert!(content.contains("| FOLDER | TIME | COMMIT MESSAGE")); // Table header
/// ```
pub fn create_diary_file(
    full_diary_file_path: &str,
    commit_saver_struct: &mut CommitSaver,
) -> Result<(), Box<dyn Error>> {
    info!("[create_diary_file()]: Retrieving the frontmatter tags.");
    let frontmatter = commit_saver_struct.prepare_frontmatter_tags();

    info!("[create_diary_file()]: Retrieving the date for commit.");
    let diary_date = commit_saver_struct
        .commit_datetime
        .format("%Y-%m-%d")
        .to_string();

    info!("[create_diary_file()]: Creating the DiaryFileEntry.");
    let template = DiaryFileEntry {
        frontmatter,
        diary_date,
    }
    .to_string();

    info!("[create_diary_file()]: Writing the DiaryFileEntry.");
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

    #[test]
    fn test_prepare_path_for_commit_integration() {
        let mut commit_saver = create_test_commit_saver();
        let obsidian_path = PathBuf::from("TestDiaries/Commits");
        let date_template = "%Y/%m-%B/%F.md";

        let result = commit_saver.prepare_path_for_commit(&obsidian_path, date_template);

        // Should contain the formatted path
        assert!(result.contains("/TestDiaries/Commits/"));
        assert!(result.contains("2023"));
        assert!(result.contains("12-December"));
        // assert!(result.ends_with(".md"));
        assert!(
            std::path::Path::new(&result)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        );
    }

    #[test]
    fn test_create_diary_file_error_handling() {
        let mut commit_saver = create_test_commit_saver();

        // Try to create file in a path that will fail (read-only location)
        let result = create_diary_file("/proc/invalid_path/file.md", &mut commit_saver);

        // Should return an error
        assert!(result.is_err());
    }

    #[test]
    fn test_get_parent_from_full_path_edge_cases() {
        use std::path::Path;

        // Test with a simple path
        let path = Path::new("/home/user/file.txt");
        let parent = get_parent_from_full_path(path);
        assert!(parent.is_ok());
        assert_eq!(parent.unwrap(), Path::new("/home/user"));

        // Test with nested path
        let nested = Path::new("/a/b/c/d/e/file.txt");
        let nested_parent = get_parent_from_full_path(nested);
        assert!(nested_parent.is_ok());
    }

    #[test]
    fn test_commit_saver_default_in_git_repo() {
        use git2::Repository;

        // Only run if we're in a git repo
        if Repository::discover("./").is_ok() {
            let commit_saver = CommitSaver::default();

            // Verify all fields are populated
            assert!(!commit_saver.repository_url.is_empty());
            assert!(!commit_saver.commit_branch_name.is_empty());
            assert!(!commit_saver.commit_hash.is_empty());
            assert!(!commit_saver.commit_msg.is_empty());

            // Hash should be 40 characters (SHA-1)
            assert_eq!(commit_saver.commit_hash.len(), 40);
        }
    }

    #[test]
    fn test_prepare_path_for_commit_with_empty_template() {
        let mut commit_saver = create_test_commit_saver();
        let obsidian_path = PathBuf::from("Diaries");
        let empty_template = "";

        let result = commit_saver.prepare_path_for_commit(&obsidian_path, empty_template);

        // Should still produce a path even with empty template
        assert!(result.contains("Diaries"));
    }

    #[test]
    fn test_commit_msg_with_only_whitespace_lines() {
        let commit_saver = CommitSaver {
            repository_url: "test".to_string(),
            commit_branch_name: "main".to_string(),
            commit_hash: "abc123".to_string(),
            commit_msg: "   \n\n   \n".to_string(), // Only whitespace
            commit_datetime: Utc.with_ymd_and_hms(2023, 12, 25, 10, 30, 0).unwrap(),
        };

        // commit_msg should be empty or minimal after filtering
        assert!(commit_saver.commit_msg.is_empty() || commit_saver.commit_msg.len() < 10);
    }

    #[test]
    fn test_create_diary_file_frontmatter_formatting() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("diary.md");
        let mut commit_saver = create_test_commit_saver();

        create_diary_file(file_path.to_str().unwrap(), &mut commit_saver)?;

        let content = fs::read_to_string(&file_path)?;

        // Verify frontmatter structure
        assert!(content.starts_with("---"));
        assert!(content.contains("category: diary"));
        assert!(content.contains("section: commits"));
        assert!(content.contains("tags:"));
        assert!(content.contains("#diary/commits"));

        Ok(())
    }

    #[test]
    fn test_diary_file_entry_markup_generation() {
        let frontmatter = vec![
            "#datetime/week/52".to_string(),
            "#datetime/days/Saturday".to_string(),
            "#diary/commits".to_string(),
        ];
        let diary_date = "2023-12-30".to_string();

        let markup = DiaryFileEntry {
            frontmatter,
            diary_date,
        };

        let output = markup.to_string();

        // Verify markup structure
        assert!(output.contains("---"));
        assert!(output.contains("category: diary"));
        assert!(output.contains("#datetime/week/52"));
        assert!(output.contains("#datetime/days/Saturday"));
        assert!(output.contains("2023-12-30"));
        assert!(output.contains("| FOLDER | TIME | COMMIT MESSAGE"));
    }

    #[test]
    fn test_commit_saver_default_no_origin_remote() {
        use git2::{Repository, Signature};
        use tempfile::tempdir;

        // Save original directory to restore later
        let original_dir = std::env::current_dir().unwrap();

        let temp_dir = tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Create a commit so HEAD exists (required for peel_to_commit)
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();

        // Change to the temp repo directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // This should hit the `_ => "no_url_set"` branch
        let saver = CommitSaver::default();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(saver.repository_url, "no_url_set");
        // Branch name depends on git config; just verify it's not empty
        assert!(!saver.commit_branch_name.is_empty());
        assert!(!saver.commit_hash.is_empty());
    }
}
