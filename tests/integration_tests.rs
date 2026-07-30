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

/// An unrecognised config section must be reported on stderr, not swallowed.
///
/// The git hook runs the binary with no `RUST_LOG`, where `env_logger` caps the
/// level at Error, so `log::warn!` alone is invisible. Without a visible
/// report, a misspelt section (`[excludes]`) silently disables exclusion and
/// the repos meant to be skipped get journalled.
#[test]
fn unknown_config_section_is_reported_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let ini = dir.path().join("rusty-commit-saver.ini");
    fs::write(&ini, ini_with_section(dir.path(), "excludes")).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rusty-commit-saver"))
        .env("RUSTY_COMMIT_SAVER_CONFIG", &ini)
        .env_remove("RUST_LOG")
        .current_dir(dir.path())
        .output()
        .expect("the binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ignoring unrecognised config sections"),
        "the unknown section was not reported; stderr was: {stderr}"
    );
    assert!(
        stderr.contains("excludes"),
        "the report did not name the section; stderr was: {stderr}"
    );
}

/// The counterpart: a config with only known sections reports nothing.
#[test]
fn known_config_sections_are_reported_silently() {
    let dir = tempfile::tempdir().unwrap();
    let ini = dir.path().join("rusty-commit-saver.ini");
    fs::write(&ini, ini_with_section(dir.path(), "exclude")).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rusty-commit-saver"))
        .env("RUSTY_COMMIT_SAVER_CONFIG", &ini)
        .env_remove("RUST_LOG")
        .current_dir(dir.path())
        .output()
        .expect("the binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Positive control first: the run must actually reach past config loading,
    // otherwise the assertion below would pass vacuously. There is no git repo
    // in the temp cwd, so the run dies right after the config is read.
    assert!(
        stderr.contains("panicked"),
        "the run did not get past config loading; stderr was: {stderr}"
    );
    assert!(
        !stderr.contains("ignoring unrecognised config sections"),
        "a fully known config must not report anything; stderr was: {stderr}"
    );
}

/// A minimal valid config whose vault lives inside `dir`, plus one extra
/// section under `extra` - `exclude` for the known case, anything else for the
/// unknown one. The vault path stays inside the temp dir so a run that gets
/// further than expected cannot write outside it.
fn ini_with_section(dir: &std::path::Path, extra: &str) -> String {
    format!(
        "[obsidian]\nroot_path_dir={}\ncommit_path=Commits\n\
         [templates]\ncommit_date_path=%Y-%m-%d.md\ncommit_datetime=%H:%M\n\
         [{extra}]\nrepos=claude-src\n",
        dir.join("vault").display()
    )
}
