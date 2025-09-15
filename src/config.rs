use std::path::PathBuf;

use clap::Parser;
use configparser::ini::Ini;
use once_cell::sync::OnceCell;

#[derive(Debug, Default)]
pub struct GlobalVars {
    pub config: OnceCell<Ini>,
    pub obsidian_root_path_dir: OnceCell<PathBuf>,
    pub obisidian_commit_path: OnceCell<PathBuf>,

    pub template_commit_date_path: OnceCell<String>,
    pub template_commit_datetime: OnceCell<String>,
}

impl GlobalVars {
    pub fn new() -> Self {
        GlobalVars {
            config: OnceCell::new(),
            obsidian_root_path_dir: OnceCell::new(),
            obisidian_commit_path: OnceCell::new(),
            template_commit_date_path: OnceCell::new(),
            template_commit_datetime: OnceCell::new(),
        }
    }

    pub fn set_all(&self) -> &Self {
        let config = get_ini_file();

        self.config
            .set(config)
            .expect("Coulnd't set config in GlobalVars");

        self.set_obsidian_vars();

        self
    }

    fn get_config(&self) -> Ini {
        self.config
            .get()
            .expect("Could not get Config. Config not initialized")
            .clone()
    }

    fn get_key_from_section_from_ini(&self, section: &str, key: &str) -> Option<String> {
        self.config
            .get()
            .expect("Retrieving the config for commit_path")
            .get(section, key)
    }

    fn get_sections_from_config(&self) -> Vec<String> {
        let sections = self.get_config().sections();
        if self.get_config().sections().len() == 2 {
            sections
        } else {
            panic!("Config INI has wrong number of sections")
        }
    }

    fn set_obsidian_vars(&self) {
        for section in self.get_sections_from_config() {
            if section == "obsidian" {
                self.set_obsidian_root_path_dir(&section);
                self.set_obsidian_commit_path(&section);
            } else if section == "templates" {
                self.set_templates_commit_date_path(&section);
                self.set_templates_datetime(&section);
            } else {
                panic!("Other sections in the INI File aren't supported")
            }
        }
    }

    fn set_templates_datetime(&self, section: &str) {
        let key = self
            .get_key_from_section_from_ini(section, "commit_datetime")
            .expect("Could not get the commit_datetime from INI");
        self.template_commit_date_path
            .set(key)
            .expect("Could not set the template_commit_datetime GlobalVars");
    }

    fn set_templates_commit_date_path(&self, section: &str) {
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

        let vec_str = string_path.split('/');

        let mut path = PathBuf::new();

        for s in vec_str {
            path.push(s);
        }
        self.obisidian_commit_path
            .set(path)
            .expect("Could not set the path for obsidian_root_path_dir");
    }

    fn set_obsidian_root_path_dir(&self, section: &str) {
        let string_path = self
            .get_key_from_section_from_ini(section, "root_path_dir")
            .expect("Could not get commit_path from config");

        let vec_str = string_path.split('/');
        let mut path = PathBuf::new();

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

pub fn retrieve_config_file_path() -> String {
    get_or_default_config_ini_path()
}

fn get_or_default_config_ini_path() -> String {
    let args = UserInput::parse();

    match args.config_ini {
        Some(cfg_str) => cfg_str,
        None => get_default_ini_path(),
    }
}

fn get_default_ini_path() -> String {
    "~/.config/rusty-commit-saver/rusty-commit-saver.ini".to_string()
}

fn get_ini_file() -> Ini {
    let config_ini_path = retrieve_config_file_path();
    let mut config = Ini::new();
    config
        .read(config_ini_path)
        .expect("Could not read the INI file!");
    config
}

#[cfg(test)]
mod global_vars_tests {
    use super::*;
    // use chrono::{DateTime, Local, TimeZone};
    // use configparser::ini::Ini;

    // Test helpers
    // fn create_test_config() -> Ini {
    //     let mut config = Ini::new();
    //     config.set("section1", "key1", Some("value1".to_string()));
    //     config
    // }

    // fn create_test_datetime() -> DateTime<Local> {
    //     Local.with_ymd_and_hms(2023, 12, 25, 10, 30, 0).unwrap()
    // }

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

    // #[test]
    // fn test_set_all_success() {
    //     let global_vars = GlobalVars::new();
    //     let config = create_test_config();
    //     let today = create_test_datetime();
    //
    //     let _result = global_vars.set_all(config.clone(), today);
    //
    //     assert!(global_vars.config.get().is_some());
    //     assert!(global_vars.today.get().is_some());
    //     assert_eq!(
    //         global_vars.config.get().unwrap().get("section1", "key1"),
    //         Some("value1".to_string())
    //     );
    // }

    // #[test]
    // #[should_panic]
    // fn test_set_all_config_already_set() {
    //     let global_vars = GlobalVars::new();
    //     let config1 = create_test_config();
    //     let config2 = create_test_config();
    //     let today = create_test_datetime();
    //
    //     global_vars.set_all(config1, today);
    //     // This should panic because config is already set
    //     global_vars.set_all(config2, today);
    // }

    // #[test]
    // #[should_panic]
    // fn test_set_all_today_already_set() {
    //     let global_vars = GlobalVars::new();
    //     let config = create_test_config();
    //     let today1 = create_test_datetime();
    //     let today2 = Local.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    //
    //     global_vars.set_all(config.clone(), today1);
    //     // This should panic because today is already set
    //     global_vars.set_all(config, today2);
    // }

    // #[test]
    // fn test_get_config_success() {
    //     let global_vars = GlobalVars::new();
    //     let config = create_test_config();
    //     let today = create_test_datetime();
    //
    //     global_vars.set_all(config.clone(), today);
    //     let result = global_vars.get_config();
    //
    //     assert!(result.is_ok());
    //     let retrieved_config = result.unwrap();
    //     assert_eq!(
    //         retrieved_config.get("section1", "key1"),
    //         Some("value1".to_string())
    //     );
    // }

    // #[test]
    // fn test_get_config_not_initialized() {
    //     let global_vars = GlobalVars::new();
    //
    //     let result = global_vars.get_config();
    //
    //     assert!(result.is_err());
    //     assert_eq!(result.unwrap_err(), "Cnofig not initialized");
    // }
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
