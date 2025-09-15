// Integration tests
#[cfg(test)]
mod integration_tests {
    use chrono::{TimeZone, Utc};
    use rusty_commit_saver::CommitSaver;
    use rusty_commit_saver::vim_commit::create_diary_file;
    use rusty_commit_saver::vim_commit::create_directories_for_new_entry;
    use std::fs;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_full_workflow_diary_creation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let mut commit_saver = CommitSaver {
            repository_url: "https://github.com/test/repo.git".to_string(),
            commit_branch_name: "feature-branch".to_string(),
            commit_hash: "abc123def456789".to_string(),
            commit_msg: "Add new feature with tests".to_string(),
            commit_datetime: Utc.with_ymd_and_hms(2023, 6, 15, 14, 30, 45).unwrap(),
        };

        // Test path preparation
        let diary_path = commit_saver.prepare_path_for_commit();
        let full_path = temp_dir.path().join(diary_path);

        // Test directory creation
        create_directories_for_new_entry(&full_path)?;

        // Test file creation
        create_diary_file(full_path.to_str().unwrap(), &mut commit_saver)?;

        // Verify file exists and has correct content
        assert!(full_path.exists());
        let content = fs::read_to_string(&full_path)?;

        assert!(content.contains("category: diary"));
        assert!(content.contains("2023-06-15"));
        assert!(content.contains("#datetime/week/24")); // Week 24 for June 15, 2023
        assert!(content.contains("#datetime/days/Thursday"));

        // Test appending entry
        commit_saver.append_entry_to_diary(&full_path)?;

        let updated_content = fs::read_to_string(&full_path)?;
        assert!(updated_content.contains("Add new feature with tests"));
        assert!(updated_content.contains("feature-branch"));
        assert!(updated_content.contains("abc123def456789"));

        Ok(())
    }

    #[test]
    fn test_commit_message_with_special_characters() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_diary.md");
        File::create(&file_path)?;

        let mut commit_saver = CommitSaver {
            repository_url: "https://github.com/test/repo.git".to_string(),
            commit_branch_name: "main".to_string(),
            commit_hash: "abc123".to_string(),
            commit_msg: "Fix: handle | pipes & ampersands < > brackets".to_string(),
            commit_datetime: Utc.with_ymd_and_hms(2023, 12, 25, 10, 30, 0).unwrap(),
        };

        commit_saver.append_entry_to_diary(&file_path)?;

        let content = fs::read_to_string(&file_path)?;
        assert!(content.contains("Fix: handle | pipes & ampersands < > brackets"));

        Ok(())
    }
}
