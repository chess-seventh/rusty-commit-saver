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
