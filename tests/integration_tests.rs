// // Integration tests
// #[cfg(test)]
// mod integration_tests {
//     use chrono::{TimeZone, Utc};
//     use rusty_commit_saver::CommitSaver;
//     use std::fs;
//     use std::fs::File;
//     use tempfile::tempdir;
//
//     #[test]
//     fn test_commit_message_with_special_characters() -> Result<(), Box<dyn std::error::Error>> {
//         let temp_dir = tempdir()?;
//         let file_path = temp_dir.path().join("test_diary.md");
//         File::create(&file_path)?;
//
//         let mut commit_saver = CommitSaver {
//             repository_url: "https://github.com/test/repo.git".to_string(),
//             commit_branch_name: "main".to_string(),
//             commit_hash: "abc123".to_string(),
//             commit_msg: "Fix: handle | pipes & ampersands < > brackets".to_string(),
//             commit_datetime: Utc.with_ymd_and_hms(2023, 12, 25, 10, 30, 0).unwrap(),
//         };
//
//         commit_saver.append_entry_to_diary(&file_path)?;
//
//         let content = fs::read_to_string(&file_path)?;
//         assert!(content.contains("Fix: handle | pipes & ampersands < > brackets"));
//
//         Ok(())
//     }
// }

use rusty_commit_saver::config::GlobalVars;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_global_vars_full_integration_workflow() {
    // Create a temporary config file
    let mut temp_config = NamedTempFile::new().unwrap();
    writeln!(temp_config, "[obsidian]").unwrap();
    writeln!(temp_config, "root_path_dir=/tmp/integration_test").unwrap();
    writeln!(temp_config, "commit_path=Integration/Test").unwrap();
    writeln!(temp_config, "[templates]").unwrap();
    writeln!(temp_config, "commit_date_path=%Y-%m-%d.md").unwrap();
    writeln!(temp_config, "commit_datetime=%Y-%m-%d %H:%M").unwrap();
    temp_config.flush().unwrap();

    // Read the config file manually and parse
    let config_content = fs::read_to_string(temp_config.path()).unwrap();
    let config = rusty_commit_saver::config::parse_ini_content(&config_content).unwrap();

    // Test the full workflow
    let global_vars = GlobalVars::new();

    // Manually set config (simulating what set_all does)
    global_vars.config.set(config).unwrap();
    global_vars.set_obsidian_vars();

    // Verify all getters work
    let root = global_vars.get_obsidian_root_path_dir();
    let commit = global_vars.get_obsidian_commit_path();
    let date_path = global_vars.get_template_commit_date_path();
    let datetime = global_vars.get_template_commit_datetime();

    assert!(root.to_string_lossy().contains("integration_test"));
    assert!(commit.to_string_lossy().contains("Integration"));
    assert_eq!(date_path, "%Y-%m-%d.md");
    assert_eq!(datetime, "%Y-%m-%d %H:%M");
}
