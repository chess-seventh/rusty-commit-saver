//!
//! Save all my commits to Obisidian
//!

pub mod vim_commit;
use vim_commit::CommitSaver;
pub mod config;
use crate::vim_commit::check_diary_path_exists;
use crate::vim_commit::create_diary_file;
use crate::vim_commit::create_directories_for_new_entry;
use config::GlobalVars;

use log::error;
use log::info;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("[main()]: Instanciating GlobalVars Struct.");
    let global_vars = GlobalVars::new();
    global_vars.set_all();

    info!("[main()]: Retrieving values from GlobalVars Struct.");
    let mut obsidian_root_path_dir = global_vars.get_obsidian_root_path_dir();
    let obsidian_commit_path = global_vars.get_obsidian_commit_path();
    let template_commit_date_path = global_vars.get_template_commit_date_path();
    let _template_commit_datetime = global_vars.get_template_commit_datetime();

    info!("[main()]: Instanciating CommitSsaver Struct");
    let mut commit_saver_struct = CommitSaver::new();

    info!("[main()]: Preparing the diary entry path to the new commit.");
    let diary_entry_path = commit_saver_struct
        .prepare_path_for_commit(&obsidian_commit_path, &template_commit_date_path);

    obsidian_root_path_dir.push(diary_entry_path);

    let tmp = &obsidian_root_path_dir.as_os_str().to_str().unwrap();

    info!("[main()] Checking if Diary file and/or path exists.");
    if let Ok(()) = check_diary_path_exists(&obsidian_root_path_dir) {
        info!("[main()] Diary file and path exists: {tmp:}");
    } else {
        info!("[main()] Diary file and or path DO NOT exist.");
        info!("[main()] Creating the directories for the new entry.");
        create_directories_for_new_entry(&obsidian_root_path_dir)?;

        info!("[main()] Creating the files for the new entry.");
        create_diary_file(
            obsidian_root_path_dir.as_os_str().to_str().unwrap(),
            &mut commit_saver_struct,
        )?;
    }

    // write commit
    info!("[main()] Logging the commit in the file.");
    match commit_saver_struct.append_entry_to_diary(&obsidian_root_path_dir) {
        Ok(()) => {
            info!("[main] Commit logged in ");
            Ok(())
        }
        Err(e) => {
            error!("[main] {e:}");
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
}

// Nextest configuration tests
// #[cfg(test)]
// mod nextest_config_tests {
//     use super::*;
//
//     // Test that can be run in parallel
//     #[test]
//     fn test_parallel_safe_function() {
//         let result = get_default_ini_path();
//         assert_eq!(
//             result,
//             "~/.config/rusty-commit-saver/rusty-commit-saver.ini"
//         );
//     }
//
//     // Test that needs to be run serially (if it modifies global state)
//     #[test]
//     #[serial_test::serial]
//     fn test_serial_required_function() {
//         // Tests that modify environment variables or global state
//         std::env::set_var("TEST_VAR", "test_value");
//         assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");
//         std::env::remove_var("TEST_VAR");
//     }
//
//     // Slow test that should be in a separate test binary
//     #[test]
//     #[ignore = "slow"]
//     fn test_slow_operation() {
//         std::thread::sleep(std::time::Duration::from_millis(100));
//         assert!(true);
//     }
// }
