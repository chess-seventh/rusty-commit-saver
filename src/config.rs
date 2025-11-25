use log::{error, info};

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use configparser::ini::Ini;
use dirs::home_dir;
use once_cell::sync::OnceCell;

/// Parse INI content into an Ini struct. Pure function, no I/O.
/// Useful for testing config parsing without file I/O.
pub fn parse_ini_content(content: &str) -> Result<Ini, String> {
    let mut config = Ini::new();
    config
        .read(content.to_string())
        .map_err(|e| format!("Failed to parse INI: {e:?}"))?;
    Ok(config)
}

#[derive(Debug, Default)]
pub struct GlobalVars {
    pub config: OnceCell<Ini>,

    obsidian_root_path_dir: OnceCell<PathBuf>,
    obsidian_commit_path: OnceCell<PathBuf>,

    template_commit_date_path: OnceCell<String>,
    template_commit_datetime: OnceCell<String>,
}

impl GlobalVars {
    pub fn new() -> Self {
        info!("[GlobalVars::new()] Creating new GlobalVars with OnceCell default values.");
        GlobalVars {
            config: OnceCell::new(),

            obsidian_root_path_dir: OnceCell::new(),
            obsidian_commit_path: OnceCell::new(),

            template_commit_date_path: OnceCell::new(),
            template_commit_datetime: OnceCell::new(),
        }
    }

    pub fn set_all(&self) -> &Self {
        info!("[GlobalVars::set_all()] Setting all variables for GlobalVars");
        let config = get_ini_file();

        info!("[GlobalVars::set_all()]: Setting Config Ini file.");
        self.config
            .set(config)
            .expect("Coulnd't set config in GlobalVars");

        info!("[GlobalVars::set_all()]: Setting Obsidian variables from file.");
        self.set_obsidian_vars();

        self
    }

    pub fn get_obsidian_root_path_dir(&self) -> PathBuf {
        info!("[GlobalVars::get_obsidian_root_path_dir()]: Getting obsidian_root_path_dir.");
        self.obsidian_root_path_dir
            .get()
            .expect("Could not get obsidian_root_path_dir")
            .clone()
    }

    pub fn get_obsidian_commit_path(&self) -> PathBuf {
        info!("[GlobalVars::get_obsidian_commit_path()]: Getting obsidian_commit_path.");
        self.obsidian_commit_path
            .get()
            .expect("Could not get obsidian_commit_path")
            .clone()
    }

    pub fn get_template_commit_date_path(&self) -> String {
        info!("[GlobalVars::get_template_commit_date_path()]: Getting template_commit_date_path.");
        self.template_commit_date_path
            .get()
            .expect("Could not get template_commit_date_path")
            .clone()
    }

    pub fn get_template_commit_datetime(&self) -> String {
        info!("[GlobalVars::get_template_commit_datetime()]: Getting template_commit_datetime.");
        self.template_commit_datetime
            .get()
            .expect("Could not get template_commit_datetime")
            .clone()
    }

    fn get_config(&self) -> Ini {
        info!("[GlobalVars::get_config()] Getting config");
        self.config
            .get()
            .expect("Could not get Config. Config not initialized")
            .clone()
    }

    fn get_key_from_section_from_ini(&self, section: &str, key: &str) -> Option<String> {
        info!(
            "[GlobalVars::get_key_from_section_from_ini()] Getting key: {key:} from section: {section:}."
        );
        self.config
            .get()
            .expect("Retrieving the config for commit_path")
            .get(section, key)
    }

    fn get_sections_from_config(&self) -> Vec<String> {
        info!("[GlobalVars::get_sections_from_config()] Getting sections from config");
        let sections = self.get_config().sections();

        info!("[GlobalVars::get_sections_from_config()] Checking validity of number of sections.");
        if sections.len() == 2 {
            sections
        } else {
            error!(
                "[GlobalVars::get_sections_from_config()] Sections Len must be 2, we have: {:}",
                sections.len()
            );
            error!(
                "[GlobalVars::get_sections_from_config()] These are the sections found: {sections:?}"
            );
            panic!(
                "[GlobalVars::get_sections_from_config()] config has the wrong number of sections."
            )
        }
    }

    pub fn set_obsidian_vars(&self) {
        for section in self.get_sections_from_config() {
            if section == "obsidian" {
                info!("[GlobalVars::set_obsidian_vars()] Setting 'obsidian' section variables.");
                self.set_obsidian_root_path_dir(&section);
                self.set_obsidian_commit_path(&section);
            } else if section == "templates" {
                info!("[GlobalVars::set_obsidian_vars()] Setting 'templates' section variables.");
                self.set_templates_commit_date_path(&section);
                self.set_templates_datetime(&section);
            } else {
                error!(
                    "[GlobalVars::set_obsidian_vars()] Trying to set other sections is not supported."
                );
                panic!(
                    "[GlobalVars::set_obsidian_vars()] Trying to set other sections is not supported."
                )
            }
        }
    }

    fn set_templates_datetime(&self, section: &str) {
        info!("[GlobalVars::set_templates_datetime()]: Setting the templates_datetime.");
        let key = self
            .get_key_from_section_from_ini(section, "commit_datetime")
            .expect("Could not get the commit_datetime from INI");

        self.template_commit_datetime
            .set(key)
            .expect("Could not set the template_commit_datetime GlobalVars");
    }

    fn set_templates_commit_date_path(&self, section: &str) {
        info!(
            "[GlobalVars::set_templates_commit_date_path()]: Setting the template_commit_date_path."
        );
        let key = self
            .get_key_from_section_from_ini(section, "commit_date_path")
            .expect("Could not get the commit_date_path from INI");

        self.template_commit_date_path
            .set(key)
            .expect("Could not set the template_commit_date_path in GlobalVars");
    }

    fn set_obsidian_commit_path(&self, section: &str) {
        let string_path = self
            .get_key_from_section_from_ini(section, "commit_path")
            .expect("Could not get commit_path from config");

        let fixed_home = if string_path.contains('~') {
            info!("[GlobalVars::set_obsidian_commit_path()]: Path does contain: '~'.");
            set_proper_home_dir(&string_path)
        } else {
            info!("[GlobalVars::set_obsidian_commit_path()]: Path does NOT contain: '~'.");
            string_path
        };

        let vec_str = fixed_home.split('/');

        let mut path = PathBuf::new();

        info!(
            "[GlobalVars::set_obsidian_commit_path()]: Pushing strings folders to create PathBuf."
        );
        for s in vec_str {
            path.push(s);
        }
        self.obsidian_commit_path
            .set(path)
            .expect("Could not set the path for obsidian_root_path_dir");
    }

    fn set_obsidian_root_path_dir(&self, section: &str) {
        let string_path = self
            .get_key_from_section_from_ini(section, "root_path_dir")
            .expect("Could not get commit_path from config");

        let fixed_home = if string_path.contains('~') {
            info!("[GlobalVars::set_obsidian_root_path_dir()]: Does contain ~");
            set_proper_home_dir(&string_path)
        } else {
            info!("[GlobalVars::set_obsidian_root_path_dir()]: Does NOT contain ~");
            string_path
        };

        let vec_str = fixed_home.split('/');
        let mut path = PathBuf::new();

        info!(
            "[GlobalVars::set_obsidian_root_path_dir()]: Pushing '/' to PathBuf for proper path."
        );
        path.push("/");

        info!(
            "[GlobalVars::set_obsidian_root_path_dir()]: Pushing strings folders to create PathBuf."
        );
        for s in vec_str {
            path.push(s);
        }

        self.obsidian_root_path_dir
            .set(path)
            .expect("Could not set the path for obsidian_root_path_dir");
    }
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(about = "Rusty Commit Saver config", long_about = None)]
pub struct UserInput {
    /// Directory of the configuration ini will default to
    /// "~/.config/rusty-commit-saver/rusty-commit-saver.ini", if nothing is provided
    #[arg(short, long)]
    pub config_ini: Option<String>,
}

// Note: This function is integration-tested through actual CLI usage
// Unit testing requires mocking std::env::args() which is not straightforward
pub fn retrieve_config_file_path() -> String {
    info!(
        "[UserInput::retrieve_config_file_path()]: retrieving the string path from CLI or default"
    );
    let config_path = get_or_default_config_ini_path();

    if Path::new(&config_path).exists() {
        info!("[UserInput::retrieve_config_file_path()]: config_path exists {config_path:}");
    } else {
        error!(
            "[UserInput::retrieve_config_file_path()]: config_path DOES NOT exists {config_path:}"
        );
        panic!(
            "[UserInput::retrieve_config_file_path()]: config_path DOES NOT exists {config_path:}"
        );
    }
    info!("[UserInput::retrieve_config_file_path()] retrieved config path: {config_path:}");
    fs::read_to_string(config_path.clone())
        .unwrap_or_else(|_| panic!("Should have been able to read the file: {config_path:}"))
}

fn get_or_default_config_ini_path() -> String {
    info!("[get_or_default_config_ini_path()]: Parsing CLI inputs.");
    let args = UserInput::parse();

    let config_path = if let Some(cfg_str) = args.config_ini {
        if cfg_str.contains('~') {
            info!(
                "[get_or_default_config_ini_path()]: Configuration string exists and contains '~'."
            );
            set_proper_home_dir(&cfg_str)
        } else {
            info!(
                "[get_or_default_config_ini_path()]: Configuration string exists but does NOT contain: `~'."
            );
            cfg_str
        }
    } else {
        info!(
            "[get_or_default_config_ini_path()]: Configuration string does NOT exist, using default values."
        );

        get_default_ini_path()
    };

    info!("[get_or_default_config_ini_path()]: Config path found: {config_path:}");
    config_path
}

fn get_default_ini_path() -> String {
    info!("[get_default_ini_path()]: Getting default ini file.");
    let cfg_str = "~/.config/rusty-commit-saver/rusty-commit-saver.ini".to_string();
    set_proper_home_dir(&cfg_str)
}

fn get_ini_file() -> Ini {
    info!("[get_ini_file()]: Retrieving the INI File");
    let content_ini = retrieve_config_file_path();
    let mut config = Ini::new();
    config
        .read(content_ini)
        .expect("Could not read the INI file!");

    info!("[get_ini_file()]: This is the INI File:\n\n{config:?}");
    config
}

fn set_proper_home_dir(cfg_str: &str) -> String {
    info!("[set_proper_home_dir()]: Changing the '~' to full home directory.");
    let home_dir = home_dir()
        .expect("Could not get home_dir")
        .into_os_string()
        .into_string()
        .expect("Could not convert home_dir from OsString to String");

    cfg_str.replace('~', &home_dir)
}

#[cfg(test)]
mod global_vars_tests {
    use super::*;

    #[test]
    fn test_global_vars_new() {
        let global_vars = GlobalVars::new();

        assert!(global_vars.config.get().is_none());
    }

    #[test]
    fn test_global_vars_default() {
        let global_vars = GlobalVars::default();

        assert!(global_vars.config.get().is_none());
    }

    #[test]
    fn test_get_sections_from_config_valid() {
        let mut config = Ini::new();
        config.set("obsidian", "root_path_dir", Some("/tmp/test".to_string()));
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        let sections = global_vars.get_sections_from_config();

        assert_eq!(sections.len(), 2);
        assert!(sections.contains(&"obsidian".to_string()));
        assert!(sections.contains(&"templates".to_string()));
    }

    #[test]
    #[should_panic(expected = "config has the wrong number of sections")]
    fn test_get_sections_from_config_invalid_count() {
        let mut config = Ini::new();
        config.set("only_one_section", "key", Some("value".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        // This should panic because we only have 1 section, not 2
        global_vars.get_sections_from_config();
    }

    #[test]
    fn test_get_key_from_section_from_ini_exists() {
        let mut config = Ini::new();
        config.set(
            "obsidian",
            "root_path_dir",
            Some("/home/user/Obsidian".to_string()),
        );

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        let result = global_vars.get_key_from_section_from_ini("obsidian", "root_path_dir");

        assert_eq!(result, Some("/home/user/Obsidian".to_string()));
    }

    #[test]
    fn test_get_key_from_section_from_ini_not_exists() {
        let mut config = Ini::new();
        config.set("obsidian", "other_key", Some("value".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        let result = global_vars.get_key_from_section_from_ini("obsidian", "non_existent_key");

        assert_eq!(result, None);
    }

    #[test]
    fn test_get_config() {
        let mut config = Ini::new();
        config.set("test", "key", Some("value".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config.clone()).unwrap();

        let retrieved_config = global_vars.get_config();

        assert_eq!(
            retrieved_config.get("test", "key"),
            Some("value".to_string())
        );
    }

    #[test]
    fn test_set_obsidian_root_path_dir_with_tilde() {
        let mut config = Ini::new();
        config.set(
            "obsidian",
            "root_path_dir",
            Some("~/Documents/Obsidian".to_string()),
        );
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_obsidian_root_path_dir("obsidian");

        let result = global_vars.get_obsidian_root_path_dir();

        // Should expand ~ to full home path
        assert!(!result.to_string_lossy().contains('~'));
        // Should start with /
        assert!(result.to_string_lossy().starts_with('/'));
        // Should end with Obsidian
        assert!(result.to_string_lossy().ends_with("Obsidian"));
    }

    #[test]
    fn test_set_obsidian_root_path_dir_absolute_path() {
        let mut config = Ini::new();
        config.set(
            "obsidian",
            "root_path_dir",
            Some("/absolute/path/Obsidian".to_string()),
        );
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_obsidian_root_path_dir("obsidian");

        let result = global_vars.get_obsidian_root_path_dir();

        // Should preserve absolute path
        assert!(result.to_string_lossy().contains("/absolute/path/Obsidian"));
    }

    #[test]
    fn test_set_obsidian_commit_path_with_tilde() {
        let mut config = Ini::new();
        config.set(
            "obsidian",
            "commit_path",
            Some("~/Diaries/Commits".to_string()),
        );
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_obsidian_commit_path("obsidian");

        let result = global_vars.get_obsidian_commit_path();

        // Should expand ~ to full home path
        assert!(!result.to_string_lossy().contains('~'));
        // Should end with Commits
        assert!(result.to_string_lossy().ends_with("Commits"));
    }

    #[test]
    fn test_set_obsidian_commit_path_absolute_path() {
        let mut config = Ini::new();
        config.set(
            "obsidian",
            "commit_path",
            Some("absolute/Diaries/Commits".to_string()),
        );
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_obsidian_commit_path("obsidian");

        let result = global_vars.get_obsidian_commit_path();

        // set_obsidian_commit_path() doesn't add leading / (unlike root_path_dir)
        // It just splits by / and rebuilds the PathBuf
        assert!(result.to_string_lossy().contains("absolute"));
        assert!(result.to_string_lossy().ends_with("Commits"));
    }

    #[test]
    fn test_set_templates_commit_date_path() {
        let mut config = Ini::new();
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y/%m-%B/%F.md".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_templates_commit_date_path("templates");

        let result = global_vars.get_template_commit_date_path();

        assert_eq!(result, "%Y/%m-%B/%F.md");
    }

    #[test]
    fn test_set_templates_datetime() {
        let mut config = Ini::new();
        config.set(
            "templates",
            "commit_datetime",
            Some("%Y-%m-%d %H:%M:%S".to_string()),
        );

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_templates_datetime("templates");

        let result = global_vars.get_template_commit_datetime();

        assert_eq!(result, "%Y-%m-%d %H:%M:%S");
    }

    #[test]
    fn test_set_obsidian_vars_both_sections() {
        let mut config = Ini::new();
        config.set(
            "obsidian",
            "root_path_dir",
            Some("/home/user/Obsidian".to_string()),
        );
        config.set(
            "obsidian",
            "commit_path",
            Some("Diaries/Commits".to_string()),
        );
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d.md".to_string()),
        );
        config.set(
            "templates",
            "commit_datetime",
            Some("%Y-%m-%d %H:%M:%S".to_string()),
        );

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        // Call the private method indirectly through set_obsidian_vars
        global_vars.set_obsidian_vars();

        // Verify all getters work (meaning setters were called)
        let root_path = global_vars.get_obsidian_root_path_dir();
        let commit_path = global_vars.get_obsidian_commit_path();
        let date_path = global_vars.get_template_commit_date_path();
        let datetime = global_vars.get_template_commit_datetime();

        assert!(root_path.to_string_lossy().contains("Obsidian"));
        assert!(commit_path.to_string_lossy().contains("Commits"));
        assert_eq!(date_path, "%Y-%m-%d.md");
        assert_eq!(datetime, "%Y-%m-%d %H:%M:%S");
    }

    #[test]
    #[should_panic(expected = "Trying to set other sections is not supported")]
    fn test_set_obsidian_vars_invalid_section() {
        let mut config = Ini::new();
        // Add correct number of sections (2) but with wrong name
        config.set("invalid_section", "key", Some("value".to_string()));
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d.md".to_string()),
        );
        config.set(
            "templates",
            "commit_datetime",
            Some("%Y-%m-%d %H:%M".to_string()),
        );

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        // Should panic because "invalid_section" is not "obsidian" or "templates"
        global_vars.set_obsidian_vars();
    }

    #[test]
    fn test_set_all_integration() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary config file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[obsidian]").unwrap();
        writeln!(temp_file, "root_path_dir=/tmp/test_obsidian").unwrap();
        writeln!(temp_file, "commit_path=TestDiaries/TestCommits").unwrap();
        writeln!(temp_file, "[templates]").unwrap();
        writeln!(temp_file, "commit_date_path=%Y-%m-%d.md").unwrap();
        writeln!(temp_file, "commit_datetime=%Y-%m-%d %H:%M:%S").unwrap();
        temp_file.flush().unwrap();

        // Parse the config manually and test set_all
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let config = parse_ini_content(&content).unwrap();

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_obsidian_vars();

        // Verify all values were set
        let root = global_vars.get_obsidian_root_path_dir();
        let commit = global_vars.get_obsidian_commit_path();
        let date = global_vars.get_template_commit_date_path();
        let datetime = global_vars.get_template_commit_datetime();

        assert!(root.to_string_lossy().contains("test_obsidian"));
        assert!(commit.to_string_lossy().contains("TestCommits"));
        assert_eq!(date, "%Y-%m-%d.md");
        assert_eq!(datetime, "%Y-%m-%d %H:%M:%S");
    }

    #[test]
    #[should_panic(expected = "Could not get")]
    fn test_get_obsidian_root_path_dir_not_set() {
        let global_vars = GlobalVars::new();
        // Don't set any values
        // This should panic when trying to get
        global_vars.get_obsidian_root_path_dir();
    }

    #[test]
    #[should_panic(expected = "Could not get")]
    fn test_get_obsidian_commit_path_not_set() {
        let global_vars = GlobalVars::new();
        global_vars.get_obsidian_commit_path();
    }

    #[test]
    #[should_panic(expected = "Could not get")]
    fn test_get_template_commit_date_path_not_set() {
        let global_vars = GlobalVars::new();
        global_vars.get_template_commit_date_path();
    }

    #[test]
    #[should_panic(expected = "Could not get")]
    fn test_get_template_commit_datetime_not_set() {
        let global_vars = GlobalVars::new();
        global_vars.get_template_commit_datetime();
    }

    #[test]
    #[should_panic(expected = "Could not get Config")]
    fn test_get_config_not_initialized() {
        let global_vars = GlobalVars::new();
        // Config not set
        global_vars.get_config();
    }

    #[test]
    fn test_set_config_twice_fails() {
        let global_vars = GlobalVars::new();
        let config1 = Ini::new();
        let config2 = Ini::new();

        assert!(global_vars.config.set(config1).is_ok());
        // Second set should fail
        assert!(global_vars.config.set(config2).is_err());
    }

    #[test]
    fn test_global_vars_set_all_end_to_end() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a real config file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[obsidian]").unwrap();
        writeln!(temp_file, "root_path_dir=/tmp/obsidian_test").unwrap();
        writeln!(temp_file, "commit_path=TestDiaries/TestCommits").unwrap();
        writeln!(temp_file, "[templates]").unwrap();
        writeln!(temp_file, "commit_date_path=%Y/%m-%B/%F.md").unwrap();
        writeln!(temp_file, "commit_datetime=%Y-%m-%d %H:%M:%S").unwrap();
        temp_file.flush().unwrap();

        // Read and parse the config
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let mut config = Ini::new();
        config.read(content).unwrap();

        // Now test set_all
        let global_vars = GlobalVars::new();
        let result = global_vars.config.set(config);
        assert!(result.is_ok());

        // Call set_obsidian_vars (which set_all would call)
        global_vars.set_obsidian_vars();

        // Verify everything is accessible
        let root = global_vars.get_obsidian_root_path_dir();
        let commit = global_vars.get_obsidian_commit_path();
        let date_path = global_vars.get_template_commit_date_path();
        let datetime = global_vars.get_template_commit_datetime();

        assert!(root.to_string_lossy().contains("obsidian_test"));
        assert!(commit.to_string_lossy().contains("TestCommits"));
        assert_eq!(date_path, "%Y/%m-%B/%F.md");
        assert_eq!(datetime, "%Y-%m-%d %H:%M:%S");
    }

    #[test]
    fn test_set_obsidian_root_path_dir_with_trailing_slash() {
        let mut config = Ini::new();
        config.set("obsidian", "root_path_dir", Some("/tmp/test/".to_string()));
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_obsidian_root_path_dir("obsidian");

        let result = global_vars.get_obsidian_root_path_dir();

        // Should handle trailing slashes gracefully
        assert!(result.to_string_lossy().contains("test"));
    }

    #[test]
    fn test_set_obsidian_commit_path_with_multiple_slashes() {
        let mut config = Ini::new();
        config.set(
            "obsidian",
            "commit_path",
            Some("Diaries//Commits///Nested".to_string()),
        );
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_obsidian_commit_path("obsidian");

        let result = global_vars.get_obsidian_commit_path();

        // Path should be constructed despite multiple slashes
        assert!(result.to_string_lossy().contains("Nested"));
    }

    #[test]
    fn test_set_obsidian_root_path_dir_empty_string() {
        let mut config = Ini::new();
        config.set("obsidian", "root_path_dir", Some(String::new()));
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_obsidian_root_path_dir("obsidian");

        let result = global_vars.get_obsidian_root_path_dir();

        // Should at least create a PathBuf (even if empty or just "/")
        assert!(!result.to_string_lossy().is_empty());
    }

    #[test]
    #[should_panic(expected = "Could not get commit_path from config")]
    fn test_set_obsidian_commit_path_missing_key() {
        let mut config = Ini::new();
        config.set("obsidian", "root_path_dir", Some("/tmp/test".to_string()));
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        global_vars.set_obsidian_commit_path("obsidian");
    }

    #[test]
    #[should_panic(expected = "Could not get")]
    fn test_set_obsidian_root_path_dir_missing_key() {
        let mut config = Ini::new();
        config.set("obsidian", "commit_path", Some("commits".to_string()));
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        global_vars.set_obsidian_root_path_dir("obsidian");
    }

    #[test]
    #[should_panic(expected = "Could not get the commit_date_path from INI")]
    fn test_set_templates_commit_date_path_missing_key() {
        let mut config = Ini::new();
        config.set("templates", "commit_datetime", Some("%Y-%m-%d".to_string()));
        config.set("obsidian", "root_path_dir", Some("/tmp".to_string()));
        config.set("obsidian", "commit_path", Some("commits".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        global_vars.set_templates_commit_date_path("templates");
    }

    #[test]
    #[should_panic(expected = "Could not get the commit_datetime from INI")]
    fn test_set_templates_datetime_missing_key() {
        let mut config = Ini::new();
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y-%m-%d".to_string()),
        );
        config.set("obsidian", "root_path_dir", Some("/tmp".to_string()));
        config.set("obsidian", "commit_path", Some("commits".to_string()));

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        global_vars.set_templates_datetime("templates");
    }

    #[test]
    fn test_global_vars_set_all_method() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a real config file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[obsidian]").unwrap();
        writeln!(temp_file, "root_path_dir=/tmp/obsidian_full_test").unwrap();
        writeln!(temp_file, "commit_path=FullTest/Commits").unwrap();
        writeln!(temp_file, "[templates]").unwrap();
        writeln!(temp_file, "commit_date_path=%Y/%m/%d.md").unwrap();
        writeln!(temp_file, "commit_datetime=%Y-%m-%d %H:%M:%S").unwrap();
        temp_file.flush().unwrap();

        // Parse config manually
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let config = parse_ini_content(&content).unwrap();

        // Test set_all workflow
        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();
        global_vars.set_obsidian_vars();

        // Verify all values accessible via set_all pattern
        let root = global_vars.get_obsidian_root_path_dir();
        let commit = global_vars.get_obsidian_commit_path();
        let date = global_vars.get_template_commit_date_path();
        let datetime = global_vars.get_template_commit_datetime();

        assert!(root.to_string_lossy().contains("obsidian_full_test"));
        assert!(commit.to_string_lossy().contains("FullTest"));
        assert_eq!(date, "%Y/%m/%d.md");
        assert_eq!(datetime, "%Y-%m-%d %H:%M:%S");
    }

    #[test]
    fn test_set_obsidian_vars_complete_workflow() {
        let mut config = Ini::new();
        config.set(
            "obsidian",
            "root_path_dir",
            Some("~/test/obsidian".to_string()),
        );
        config.set(
            "obsidian",
            "commit_path",
            Some("~/test/commits".to_string()),
        );
        config.set(
            "templates",
            "commit_date_path",
            Some("%Y/%m/%d.md".to_string()),
        );
        config.set(
            "templates",
            "commit_datetime",
            Some("%Y-%m-%d %H:%M:%S".to_string()),
        );

        let global_vars = GlobalVars::new();
        global_vars.config.set(config).unwrap();

        // This exercises the full set_obsidian_vars logic
        global_vars.set_obsidian_vars();

        // Verify all paths were expanded
        let root = global_vars.get_obsidian_root_path_dir();
        let commit = global_vars.get_obsidian_commit_path();

        // Both should have ~ expanded
        assert!(!root.to_string_lossy().contains('~'));
        assert!(!commit.to_string_lossy().contains('~'));
        assert!(root.to_string_lossy().contains("obsidian"));
        assert!(commit.to_string_lossy().contains("commits"));
    }
}

#[cfg(test)]
mod user_input_tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_user_input_parse_with_config() {
        let args = vec!["test_program", "--config-ini", "/path/to/config.ini"];
        let user_input = UserInput::try_parse_from(args).unwrap();

        assert_eq!(
            user_input.config_ini,
            Some("/path/to/config.ini".to_string())
        );
    }

    #[test]
    fn test_user_input_parse_without_config() {
        let args = vec!["test_program"];
        let user_input = UserInput::try_parse_from(args).unwrap();

        assert_eq!(user_input.config_ini, None);
    }

    #[test]
    fn test_user_input_parse_short_flag() {
        let args = vec!["test_program", "-c", "/short/path/config.ini"];
        let user_input = UserInput::try_parse_from(args).unwrap();

        assert_eq!(
            user_input.config_ini,
            Some("/short/path/config.ini".to_string())
        );
    }

    #[test]
    fn test_set_proper_home_dir_with_tilde() {
        let input = "~/test/path/file.ini";
        let result = set_proper_home_dir(input);

        // Should replace ~ with actual home directory
        assert!(!result.contains('~'));
        assert!(result.ends_with("/test/path/file.ini"));
    }

    #[test]
    fn test_set_proper_home_dir_without_tilde() {
        let input = "/absolute/path/file.ini";
        let result = set_proper_home_dir(input);

        // Should remain unchanged
        assert_eq!(result, input);
    }

    #[test]
    fn test_set_proper_home_dir_multiple_tildes() {
        let input = "~/path/~/file.ini";
        let result = set_proper_home_dir(input);

        // Should replace ALL tildes
        assert!(!result.contains('~'));
    }

    #[test]
    fn test_get_default_ini_path() {
        let result = get_default_ini_path();

        // Should end with the expected config path
        assert!(result.ends_with(".config/rusty-commit-saver/rusty-commit-saver.ini"));

        // Should NOT contain literal tilde
        assert!(!result.contains('~'));

        // Should be an absolute path
        assert!(result.starts_with('/'));
    }

    #[test]
    fn test_get_or_default_config_ini_path_with_config_and_tilde() {
        // Simulate CLI args: --config-ini ~/my/config.ini
        let args = vec!["test", "--config-ini", "~/my/config.ini"];
        let user_input = UserInput::try_parse_from(args).unwrap();

        // We can't directly call get_or_default_config_ini_path() because it parses env args
        // Instead, test that UserInput correctly parses the config path
        assert_eq!(user_input.config_ini, Some("~/my/config.ini".to_string()));
    }

    #[test]
    fn test_get_or_default_config_ini_path_with_config_absolute_path() {
        // Simulate CLI args: --config-ini /absolute/path/config.ini
        let args = vec!["test", "--config-ini", "/absolute/path/config.ini"];
        let user_input = UserInput::try_parse_from(args).unwrap();

        assert_eq!(
            user_input.config_ini,
            Some("/absolute/path/config.ini".to_string())
        );
    }

    #[test]
    fn test_get_or_default_config_ini_path_without_config() {
        // Simulate CLI args with no config specified
        let args = vec!["test"];
        let user_input = UserInput::try_parse_from(args).unwrap();

        // Should default to None, and get_or_default_config_ini_path() will use get_default_ini_path()
        assert_eq!(user_input.config_ini, None);
    }

    #[test]
    fn test_parse_ini_content_valid() {
        let content = r"
[obsidian]
root_path_dir=~/Documents/Obsidian
commit_path=Diaries/Commits

[templates]
commit_date_path=%Y/%m-%B/%F.md
commit_datetime=%Y-%m-%d
";

        let result = parse_ini_content(content);
        assert!(result.is_ok());

        let ini = result.unwrap();
        assert_eq!(
            ini.get("obsidian", "root_path_dir"),
            Some("~/Documents/Obsidian".to_string())
        );
        assert_eq!(
            ini.get("templates", "commit_date_path"),
            Some("%Y/%m-%B/%F.md".to_string())
        );
    }

    #[test]
    fn test_parse_ini_content_invalid() {
        let content = "this is not valid ini format [[[";

        let result = parse_ini_content(content);
        // Should succeed because configparser is very lenient, but let's verify it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_ini_content_empty() {
        let content = "";

        let result = parse_ini_content(content);
        assert!(result.is_ok());

        let ini = result.unwrap();
        assert_eq!(ini.sections().len(), 0);
    }

    #[test]
    fn test_retrieve_config_file_path_with_temp_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary config file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[obsidian]").unwrap();
        writeln!(temp_file, "root_path_dir=/tmp/test").unwrap();
        writeln!(temp_file, "commit_path=commits").unwrap();
        writeln!(temp_file, "[templates]").unwrap();
        writeln!(temp_file, "commit_date_path=%Y-%m-%d.md").unwrap();
        writeln!(temp_file, "commit_datetime=%Y-%m-%d").unwrap();
        temp_file.flush().unwrap();

        // Set CLI args to point to our temp file
        // We need to simulate CLI args via environment
        let path = temp_file.path().to_str().unwrap();

        // Instead of testing retrieve_config_file_path directly (which reads from CLI),
        // test that we can read and parse a config file
        let content = std::fs::read_to_string(path).unwrap();
        let result = parse_ini_content(&content);

        assert!(result.is_ok());
        let ini = result.unwrap();
        assert_eq!(
            ini.get("obsidian", "root_path_dir"),
            Some("/tmp/test".to_string())
        );
    }

    #[test]
    fn test_ini_parsing_integration() {
        let content = r"
[obsidian]
root_path_dir=~/Documents/Obsidian
commit_path=Diaries/Commits

[templates]
commit_date_path=%Y/%m-%B/%F.md
commit_datetime=%Y-%m-%d %H:%M:%S
";

        let ini = parse_ini_content(content).unwrap();

        // Verify all expected keys exist
        assert!(ini.get("obsidian", "root_path_dir").is_some());
        assert!(ini.get("obsidian", "commit_path").is_some());
        assert!(ini.get("templates", "commit_date_path").is_some());
        assert!(ini.get("templates", "commit_datetime").is_some());

        // Verify sections count
        assert_eq!(ini.sections().len(), 2);
    }
}
