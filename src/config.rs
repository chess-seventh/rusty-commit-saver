use chrono::{DateTime, Local};
use clap::Parser;
use configparser::ini::Ini;
use once_cell::sync::OnceCell;

#[derive(Debug, Clone, Default)]
pub struct GlobalVars {
    pub config: OnceCell<Ini>,
    pub today: OnceCell<DateTime<Local>>,
}

impl GlobalVars {
    pub fn new() -> Self {
        GlobalVars {
            config: OnceCell::new(),
            today: OnceCell::new(),
        }
    }

    pub fn set_all(&self, config: Ini, today: DateTime<Local>) -> &Self {
        self.config
            .set(config)
            .expect("Coulnd't set config in GlobalVars");
        self.today
            .set(today)
            .expect("Couldn't set today in GlobalVars");
        self
    }

    pub fn get_config(&self) -> Result<Ini, &str> {
        self.config.get().cloned().ok_or("Cnofig not initialized")
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

#[cfg(test)]
mod global_vars_tests {
    use super::*;
    use chrono::{DateTime, Local, TimeZone};
    use configparser::ini::Ini;
    // use std::fs;
    // use std::path::PathBuf;
    // use tempfile::TempDir;

    // Test helpers
    fn create_test_config() -> Ini {
        let mut config = Ini::new();
        config.set("section1", "key1", Some("value1".to_string()));
        config
    }

    fn create_test_datetime() -> DateTime<Local> {
        Local.with_ymd_and_hms(2023, 12, 25, 10, 30, 0).unwrap()
    }

    #[test]
    fn test_global_vars_new() {
        let global_vars = GlobalVars::new();

        assert!(global_vars.config.get().is_none());
        assert!(global_vars.today.get().is_none());
    }

    #[test]
    fn test_global_vars_default() {
        let global_vars = GlobalVars::default();

        assert!(global_vars.config.get().is_none());
        assert!(global_vars.today.get().is_none());
    }

    #[test]
    fn test_set_all_success() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = create_test_datetime();

        let _result = global_vars.set_all(config.clone(), today);

        assert!(global_vars.config.get().is_some());
        assert!(global_vars.today.get().is_some());
        assert_eq!(
            global_vars.config.get().unwrap().get("section1", "key1"),
            Some("value1".to_string())
        );
    }

    #[test]
    #[should_panic]
    fn test_set_all_config_already_set() {
        let global_vars = GlobalVars::new();
        let config1 = create_test_config();
        let config2 = create_test_config();
        let today = create_test_datetime();

        global_vars.set_all(config1, today);
        // This should panic because config is already set
        global_vars.set_all(config2, today);
    }

    #[test]
    #[should_panic]
    fn test_set_all_today_already_set() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today1 = create_test_datetime();
        let today2 = Local.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        global_vars.set_all(config.clone(), today1);
        // This should panic because today is already set
        global_vars.set_all(config, today2);
    }

    #[test]
    fn test_get_config_success() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = create_test_datetime();

        global_vars.set_all(config.clone(), today);
        let result = global_vars.get_config();

        assert!(result.is_ok());
        let retrieved_config = result.unwrap();
        assert_eq!(
            retrieved_config.get("section1", "key1"),
            Some("value1".to_string())
        );
    }

    #[test]
    fn test_get_config_not_initialized() {
        let global_vars = GlobalVars::new();

        let result = global_vars.get_config();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cnofig not initialized");
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
}
