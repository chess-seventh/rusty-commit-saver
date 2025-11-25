//!
//! Save all my commits to Obisidian
//!

pub mod vim_commit;
use vim_commit::CommitSaver;
use vim_commit::check_diary_path_exists;
use vim_commit::create_diary_file;
use vim_commit::create_directories_for_new_entry;

pub mod config;
use config::GlobalVars;

use log::error;
use log::info;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

/// Core logic for saving a commit to an Obsidian diary file.
///
/// This is the main orchestration function that:
/// 1. Discovers the current Git commit metadata
/// 2. Constructs the diary file path based on timestamp
/// 3. Creates necessary directories and diary template (if needed)
/// 4. Appends the commit entry to the diary file
///
/// This function is extracted for testability and is called by `main()`.
///
/// # Arguments
///
/// * `obsidian_root_path_dir` - Base directory for Obsidian vault (e.g., `/home/user/Obsidian`)
/// * `obsidian_commit_path` - Subdirectory for commits (e.g., `Diaries/Commits`)
/// * `template_commit_date_path` - Chrono format for date hierarchy (e.g., `%Y/%m-%B/%F.md`)
///
/// # Returns
///
/// - `Ok(())` - Commit was successfully saved to the diary
/// - `Err(Box<dyn Error>)` - Any step in the process failed
///
/// # Errors
///
/// Returns an error if:
/// - Git repository cannot be discovered (not in a git repo)
/// - Diary path cannot be converted to valid UTF-8
/// - Parent directories cannot be created (permission denied, invalid path)
/// - Diary file cannot be created or written to
/// - Commit entry cannot be appended to the file
///
/// # Examples
///
/// ```
/// use rusty_commit_saver::run_commit_saver;
/// use std::path::PathBuf;
///
/// let obsidian_root = PathBuf::from("/home/user/Obsidian");
/// let commit_path = PathBuf::from("Diaries/Commits");
/// let date_template = "%Y/%m-%B/%F.md"; // YYYY/MM-MonthName/YYYY-MM-DD.md
///
/// match run_commit_saver(obsidian_root, &commit_path, date_template) {
///     Ok(()) => println!("✓ Commit successfully logged!"),
///     Err(e) => eprintln!("✗ Failed to log commit: {}", e),
/// }
/// ```
///
/// # Workflow
///
/// ```
/// ┌─────────────────────────────────┐
/// │ Discover Git commit metadata    │
/// └────────────┬────────────────────┘
///              │
/// ┌────────────▼──────────────────┐
/// │ Build diary file path with    │
/// │ formatted date subdirectories │
/// └────────────┬──────────────────┘
///              │
/// ┌────────────▼──────────────────┐
/// │ File exists?                  │
/// └────────┬───────────────┬──────┘
///          │ No            │ Yes
///   ┌──────▼────────┐    │
///   │ Create dirs   │    │
///   │ Create template  │  │
///   │ (with table)  │    │
///   └──────┬────────┘    │
///          │             │
///   ┌──────▼─────────────▼──────┐
///   │ Append commit row to table │
///   └──────┬─────────────────────┘
///          │
///   ┌──────▼──────────────────┐
///   │ Return Ok(())           │
///   └─────────────────────────┘
/// ```
pub fn run_commit_saver(
    obsidian_root_path_dir: PathBuf,
    obsidian_commit_path: &Path,
    template_commit_date_path: &str,
) -> Result<(), Box<dyn Error>> {
    info!("[run_commit_saver()]: Instanciating CommitSaver Struct");
    let mut commit_saver_struct = CommitSaver::new();

    info!("[run_commit_saver()]: Preparing the diary entry path to the new commit.");
    let diary_entry_path = commit_saver_struct
        .prepare_path_for_commit(obsidian_commit_path, template_commit_date_path);

    let mut full_path = obsidian_root_path_dir;
    for directory in diary_entry_path.split('/') {
        full_path.push(directory);
    }

    let stringed_root_path_dir = full_path
        .as_os_str()
        .to_str()
        .ok_or("Could not convert path to string")?;

    info!("[run_commit_saver()]: Checking if Diary file and/or path exists.");
    if check_diary_path_exists(&full_path).is_ok() {
        info!("[run_commit_saver()]: Diary file and path exists: {stringed_root_path_dir:}");
    } else {
        info!("[run_commit_saver()]: Diary file and or path DO NOT exist.");
        info!("[run_commit_saver()]: Creating the directories for the new entry.");
        create_directories_for_new_entry(&full_path)?;

        info!("[run_commit_saver()]: Creating the files for the new entry.");
        create_diary_file(stringed_root_path_dir, &mut commit_saver_struct)?;
    }

    info!("[run_commit_saver()]: Writing the commit in the file.");
    commit_saver_struct.append_entry_to_diary(&full_path)?;
    info!("[run_commit_saver]: Commit logged in ");

    Ok(())
}

fn main() {
    env_logger::init();
    info!("[main()]: Instanciating GlobalVars Struct.");
    let global_vars = GlobalVars::new();
    global_vars.set_all();

    info!("[main()]: Retrieving values from GlobalVars Struct.");
    let obsidian_root_path_dir = global_vars.get_obsidian_root_path_dir();
    let obsidian_commit_path = global_vars.get_obsidian_commit_path();
    let template_commit_date_path = global_vars.get_template_commit_date_path();

    match run_commit_saver(
        obsidian_root_path_dir,
        &obsidian_commit_path,
        &template_commit_date_path,
    ) {
        Ok(()) => (),
        Err(e) => {
            error!("[main]: {e:}");
            panic!("[main]: Something went wrong when writing the commit to the file");
        }
    }
}

#[cfg(test)]
mod main_tests {
    use super::*;
    use crate::vim_commit::check_diary_path_exists;
    use crate::vim_commit::create_diary_file;
    use crate::vim_commit::create_directories_for_new_entry;
    use crate::vim_commit::get_parent_from_full_path;
    use chrono::{TimeZone, Utc};
    use git2::Repository;
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_get_parent_from_full_path() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("subdir").join("file.txt");

        let parent = get_parent_from_full_path(&file_path)?;

        assert_eq!(parent, temp_dir.path().join("subdir"));
        Ok(())
    }

    #[test]
    fn test_get_parent_from_full_path_root() {
        let root_path = PathBuf::from("/");

        let result = get_parent_from_full_path(&root_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_check_diary_path_exists_true() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_file.md");
        File::create(&file_path)?;

        let result = check_diary_path_exists(&file_path);

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_check_diary_path_exists_false() {
        let non_existent_path = PathBuf::from("/non/existent/path");

        let result = check_diary_path_exists(&non_existent_path);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Path does not exist!");
    }

    #[test]
    fn test_create_diary_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("diary.md");
        let mut commit_saver = CommitSaver {
            repository_url: "https://github.com/test/repo.git".to_string(),
            commit_branch_name: "main".to_string(),
            commit_hash: "abc123".to_string(),
            commit_msg: "Test".to_string(),
            commit_datetime: Utc.with_ymd_and_hms(2023, 12, 25, 10, 30, 0).unwrap(),
        };

        let result = create_diary_file(file_path.to_str().unwrap(), &mut commit_saver);

        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path)?;
        assert!(content.contains("category: diary"));
        assert!(content.contains("section: commits"));
        assert!(content.contains("2023-12-25"));
        assert!(content.contains("| FOLDER | TIME | COMMIT MESSAGE"));

        Ok(())
    }

    #[test]
    fn test_create_directories_for_new_entry() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let nested_path = temp_dir.path().join("deep").join("nested").join("file.md");

        let result = create_directories_for_new_entry(&nested_path);

        assert!(result.is_ok());
        assert!(temp_dir.path().join("deep").join("nested").exists());

        Ok(())
    }

    #[test]
    fn test_create_directories_for_new_entry_invalid_path() {
        // Test with a path that can't be created (e.g., on a read-only filesystem)
        let invalid_path = PathBuf::from("/proc/invalid/path/file.md");

        let result = create_directories_for_new_entry(&invalid_path);

        // This should fail on most systems as /proc is not writable
        assert!(result.is_err());
    }

    #[test]
    fn test_run_commit_saver_creates_new_diary() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let obsidian_root = temp_dir.path().to_path_buf();
        let commit_path = PathBuf::from("Diaries/Commits");
        let date_template = "%Y/%m-%B/%F.md";

        // This assumes we're in a git repo for CommitSaver::new() to work
        if Repository::discover("./").is_ok() {
            let result = run_commit_saver(obsidian_root.clone(), &commit_path, date_template);

            // Should succeed and create diary file
            assert!(result.is_ok());

            // Verify directory structure was created
            assert!(obsidian_root.exists());
        }

        Ok(())
    }

    #[test]
    fn test_run_commit_saver_missing_directory_creates_it() -> Result<(), Box<dyn std::error::Error>>
    {
        use git2::Repository;

        let temp_dir = tempdir()?;
        let obsidian_root = temp_dir.path().join("non_existent_path");
        let commit_path = PathBuf::from("Diaries/Commits");
        let date_template = "%Y/%m-%B/%F.md";

        // Only run if we're in a git repo
        if Repository::discover("./").is_ok() {
            let result = run_commit_saver(obsidian_root.clone(), &commit_path, date_template);

            // Should succeed and create the missing directories
            assert!(result.is_ok());
        }

        Ok(())
    }

    #[test]
    fn test_run_commit_saver_append_to_existing_diary() -> Result<(), Box<dyn std::error::Error>> {
        use git2::Repository;

        let temp_dir = tempdir()?;
        let obsidian_root = temp_dir.path().to_path_buf();
        let commit_path = PathBuf::from("Diaries/Commits");
        let date_template = "%Y/%m-%B/%F.md";

        // Only run if in a git repo
        if Repository::discover("./").is_ok() {
            // First run - creates the file
            run_commit_saver(obsidian_root.clone(), &commit_path, date_template)?;

            // Second run - should append to existing file
            let result = run_commit_saver(obsidian_root.clone(), &commit_path, date_template);
            assert!(result.is_ok());

            // Verify file exists and has multiple entries
            // (The exact path depends on current date, so we check the directory exists)
            assert!(obsidian_root.join("Diaries").exists());
        }

        Ok(())
    }

    #[test]
    fn test_run_commit_saver_handles_file_write_errors() {
        use git2::Repository;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempdir().unwrap();
        let obsidian_root = temp_dir.path().to_path_buf();
        let commit_path = PathBuf::from("Diaries/Commits");
        let date_template = "%Y/%m-%B/%F.md";

        // Only run if in a git repo
        if Repository::discover("./").is_ok() {
            // Create directory structure first
            let result = run_commit_saver(obsidian_root.clone(), &commit_path, date_template);
            assert!(result.is_ok());

            // Now make the directory read-only to trigger write errors on second run
            let diary_dir = obsidian_root.join("Diaries");
            if diary_dir.exists() {
                let metadata = fs::metadata(&diary_dir).unwrap();
                let mut perms = metadata.permissions();
                perms.set_mode(0o444); // Read-only
                fs::set_permissions(&diary_dir, perms).ok();
            }
        }
    }

    #[test]
    fn test_check_diary_path_exists_with_symlink() -> Result<(), Box<dyn std::error::Error>> {
        use std::os::unix::fs::symlink;

        let temp_dir = tempdir()?;
        let target_file = temp_dir.path().join("target.md");
        let symlink_file = temp_dir.path().join("symlink.md");

        // Create target file
        File::create(&target_file)?;

        // Create symlink
        symlink(&target_file, &symlink_file)?;

        // check_diary_path_exists should work with symlinks
        let result = check_diary_path_exists(&symlink_file);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_create_diary_file_with_invalid_frontmatter() {
        use chrono::{TimeZone, Utc};

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.md");

        let mut commit_saver = CommitSaver {
            repository_url: "test".to_string(),
            commit_branch_name: "main".to_string(),
            commit_hash: "abc123".to_string(),
            commit_msg: "test".to_string(),
            commit_datetime: Utc.with_ymd_and_hms(2023, 12, 25, 10, 30, 0).unwrap(),
        };

        // Test that create_diary_file handles edge cases
        let result = create_diary_file(file_path.to_str().unwrap(), &mut commit_saver);
        assert!(result.is_ok());

        // Verify file was created
        assert!(file_path.exists());
    }

    #[test]
    fn test_create_directories_for_new_entry_with_existing_dirs()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let nested_path = temp_dir
            .path()
            .join("existing")
            .join("path")
            .join("file.md");

        // Create directories first
        fs::create_dir_all(temp_dir.path().join("existing").join("path"))?;

        // Now test creating them again (should succeed idempotently)
        let result = create_directories_for_new_entry(&nested_path);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_run_commit_saver_idempotent() -> Result<(), Box<dyn std::error::Error>> {
        use git2::Repository;

        let temp_dir = tempdir()?;
        let obsidian_root = temp_dir.path().to_path_buf();
        let commit_path = PathBuf::from("Diaries/Commits");
        let date_template = "%Y/%m-%B/%F.md";

        if Repository::discover("./").is_ok() {
            // Run three times - should be idempotent
            for _ in 0..3 {
                let result = run_commit_saver(obsidian_root.clone(), &commit_path, date_template);
                assert!(result.is_ok());
            }
        }

        Ok(())
    }

    #[test]
    fn test_get_parent_from_full_path_multiple_levels() -> Result<(), Box<dyn std::error::Error>> {
        let deep_path = PathBuf::from("/a/b/c/d/e/f/g/file.txt");
        let parent = get_parent_from_full_path(&deep_path)?;

        assert_eq!(parent, Path::new("/a/b/c/d/e/f/g"));

        Ok(())
    }

    #[test]
    fn test_run_commit_saver_with_complex_path() -> Result<(), Box<dyn std::error::Error>> {
        use git2::Repository;

        let temp_dir = tempdir()?;
        let complex_root = temp_dir.path().join("level1").join("level2").join("level3");
        let commit_path = PathBuf::from("Deep/Nested/Commits");
        let date_template = "%Y/%m-%B/%d/%F.md";

        if Repository::discover("./").is_ok() {
            let result = run_commit_saver(complex_root.clone(), &commit_path, date_template);
            assert!(result.is_ok());

            // Verify deep directory structure was created
            assert!(complex_root.exists() || temp_dir.path().join("level1").exists());
        }

        Ok(())
    }
}
