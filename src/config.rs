use log::{error, info};

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use configparser::ini::Ini;
use dirs::home_dir;
use once_cell::sync::OnceCell;

#[derive(Debug, Default)]
pub struct GlobalVars {
    config: OnceCell<Ini>,

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
            .to_string()
    }

    pub fn get_template_commit_datetime(&self) -> String {
        info!("[GlobalVars::get_template_commit_datetime()]: Getting template_commit_datetime.");
        self.template_commit_datetime
            .get()
            .expect("Could not get template_commit_datetime")
            .to_string()
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

    fn set_obsidian_vars(&self) {
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
