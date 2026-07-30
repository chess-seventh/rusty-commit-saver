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

/// An unrecognised config key must be reported on stderr too.
///
/// The key twin of `unknown_config_section_is_reported_on_stderr`. A misspelt
/// key used to apply nothing and say nothing at all: the binary only ever asks
/// for the keys it knows, so one nobody asks for is invisible.
#[test]
fn unknown_config_key_is_reported_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let ini = dir.path().join("rusty-commit-saver.ini");
    fs::write(&ini, ini_with_extra_key(dir.path(), "commit_datetimes=%T")).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rusty-commit-saver"))
        .env("RUSTY_COMMIT_SAVER_CONFIG", &ini)
        .env_remove("RUST_LOG")
        .current_dir(dir.path())
        .output()
        .expect("the binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ignoring unrecognised config keys"),
        "the unknown key was not reported; stderr was: {stderr}"
    );
    assert!(
        stderr.contains("[templates] commit_datetimes"),
        "the report did not name the key and its section; stderr was: {stderr}"
    );
}

/// The counterpart: a config whose keys are all known reports nothing.
#[test]
fn known_config_keys_are_reported_silently() {
    let dir = tempfile::tempdir().unwrap();
    let ini = dir.path().join("rusty-commit-saver.ini");
    fs::write(&ini, ini_with_extra_key(dir.path(), "")).unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rusty-commit-saver"))
        .env("RUSTY_COMMIT_SAVER_CONFIG", &ini)
        .env_remove("RUST_LOG")
        .current_dir(dir.path())
        .output()
        .expect("the binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Positive control: the run must have read the config and got past it,
    // otherwise "reported nothing" would pass vacuously. There is no git repo
    // in the temp cwd, so it dies in repo discovery - after config loading,
    // and with a message no config fault produces.
    assert!(
        stderr.contains("failed to build CommitSaver"),
        "the run did not get past config loading; stderr was: {stderr}"
    );
    assert!(
        !stderr.contains("ignoring unrecognised config keys"),
        "a fully known config must not report anything; stderr was: {stderr}"
    );
}

/// A missing required key must name the file to edit and the key itself.
///
/// The message is the whole deliverable here: the old one named a key and a
/// source line, so diagnosing it meant reading the source.
#[test]
fn missing_required_key_names_the_file_and_the_key() {
    let dir = tempfile::tempdir().unwrap();
    let ini = dir.path().join("rusty-commit-saver.ini");
    // The rename in full: the new key is present, the old one is gone.
    fs::write(
        &ini,
        format!(
            "[obsidian]\nroot_path_dir={}\ncommit_paths=Commits\n\
             [templates]\ncommit_date_path=%Y-%m-%d.md\ncommit_datetime=%H:%M\n",
            dir.path().join("vault").display()
        ),
    )
    .unwrap();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rusty-commit-saver"))
        .env("RUSTY_COMMIT_SAVER_CONFIG", &ini)
        .env_remove("RUST_LOG")
        .current_dir(dir.path())
        .output()
        .expect("the binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&ini.display().to_string()),
        "the message must name the config file to edit; stderr was: {stderr}"
    );
    assert!(
        stderr.contains("missing required key 'commit_path' in section [obsidian]"),
        "the message must name the key and its section; stderr was: {stderr}"
    );
    assert!(
        stderr.contains("unrecognised in [obsidian]: commit_paths"),
        "the message must name the typo that explains the absence; stderr was: {stderr}"
    );
}

/// A format `chrono` cannot render must fail the same way, and before writing.
///
/// This one used to surface from inside the writer as `a formatting trait
/// implementation returned an error`, naming neither file nor key - and only
/// after an empty diary file had been created.
///
/// Runs inside a **real git repository**, unlike its neighbours here. The
/// others can die in repo discovery and still prove their point; this one
/// cannot, because "wrote nothing" is only meaningful if the run could have
/// got far enough to write. In a bare temp directory that assertion passes
/// even with the config check deleted.
#[test]
fn unrenderable_time_format_names_the_key_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("vault");
    let ini = dir.path().join("rusty-commit-saver.ini");
    fs::write(
        &ini,
        format!(
            "[obsidian]\nroot_path_dir={}\ncommit_path=Commits\n\
             [templates]\ncommit_date_path=%Y-%m-%d.md\ncommit_datetime=%Q\n",
            vault.display()
        ),
    )
    .unwrap();
    commit_once_in(dir.path());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rusty-commit-saver"))
        .env("RUSTY_COMMIT_SAVER_CONFIG", &ini)
        .env_remove("RUST_LOG")
        .current_dir(dir.path())
        .output()
        .expect("the binary should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&ini.display().to_string()),
        "the message must name the config file to edit; stderr was: {stderr}"
    );
    assert!(
        stderr.contains("key 'commit_datetime' in section [templates]"),
        "the message must name the key and its section; stderr was: {stderr}"
    );
    assert!(
        stderr.contains("'%Q'"),
        "the message must quote the value that fails; stderr was: {stderr}"
    );
    assert!(
        !vault.exists(),
        "the run must stop before creating anything; the empty diary file this \
         used to leave behind is half the reason the check exists"
    );
}

/// The control for the test above: the same run, with a format `chrono` can
/// render, must get as far as creating the vault. Without this, "wrote
/// nothing" could pass for a reason that has nothing to do with the config
/// check. What lands *in* the file is covered by the row tests in
/// `src/vim_commit.rs`.
#[test]
fn a_renderable_time_format_reaches_the_vault() {
    let dir = tempfile::tempdir().unwrap();
    let vault = dir.path().join("vault");
    let ini = dir.path().join("rusty-commit-saver.ini");
    fs::write(
        &ini,
        format!(
            "[obsidian]\nroot_path_dir={}\ncommit_path=Commits\n\
             [templates]\ncommit_date_path=%Y-%m-%d.md\ncommit_datetime=%H:%M\n",
            vault.display()
        ),
    )
    .unwrap();
    commit_once_in(dir.path());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rusty-commit-saver"))
        .env("RUSTY_COMMIT_SAVER_CONFIG", &ini)
        .env_remove("RUST_LOG")
        .current_dir(dir.path())
        .output()
        .expect("the binary should run");

    assert!(
        vault.exists(),
        "a good config must journal; stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Initialises a git repository at `path` with one commit, so a run started
/// there gets past repository discovery.
fn commit_once_in(path: &std::path::Path) {
    use git2::{Repository, Signature};

    let repo = Repository::init(path).unwrap();
    let sig = Signature::now("Test User", "test@example.com").unwrap();
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();
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

/// The same minimal config, with one extra line appended to `[templates]` -
/// an unrecognised key for the unknown case, empty for the known one.
fn ini_with_extra_key(dir: &std::path::Path, extra: &str) -> String {
    format!(
        "[obsidian]\nroot_path_dir={}\ncommit_path=Commits\n\
         [templates]\ncommit_date_path=%Y-%m-%d.md\ncommit_datetime=%H:%M\n{extra}\n",
        dir.join("vault").display()
    )
}
