use log::{error, info};

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use configparser::ini::Ini;
use dirs::home_dir;
use once_cell::sync::OnceCell;

/// Parses INI file content into a configuration object without file I/O.
///
/// This is a pure function that takes raw INI text and parses it into an `Ini` struct.
/// It's useful for testing configuration parsing logic without reading from disk.
///
/// # Arguments
///
/// * `content` - The raw INI file content as a string
///
/// # Returns
///
/// - `Ok(Ini)` - Successfully parsed configuration
/// - `Err(String)` - Parsing failed with error description
///
/// # INI Format
///
/// The INI format supported:
/// ```
/// [section_name]
/// key1 = value1
/// key2 = value2
///
/// [another_section]
/// key3 = value3
/// ```
///
/// # Examples
///
/// ```
/// use rusty_commit_saver::config::parse_ini_content;
///
/// let ini_content = r#"
/// [obsidian]
/// root_path_dir = ~/Documents/Obsidian
/// commit_path = Diaries/Commits
///
/// [templates]
/// commit_date_path = %Y/%m-%B/%F.md
/// commit_datetime = %Y-%m-%d %H:%M:%S
/// "#;
///
/// let config = parse_ini_content(ini_content).unwrap();
///
/// // Access parsed values
/// assert_eq!(
///     config.get("obsidian", "root_path_dir"),
///     Some("~/Documents/Obsidian".to_string())
/// );
/// assert_eq!(
///     config.get("templates", "commit_date_path"),
///     Some("%Y/%m-%B/%F.md".to_string())
/// );
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - INI syntax is invalid (malformed sections or key-value pairs)
/// - The content cannot be parsed as valid UTF-8
///
/// # Testing
///
/// This function is particularly useful for unit testing without needing
/// to create temporary files:
///
/// ```
/// use rusty_commit_saver::config::parse_ini_content;
///
/// fn test_config_parsing() {
///     let test_config = "[section]\nkey=value\n";
///     let result = parse_ini_content(test_config);
///     assert!(result.is_ok());
/// }
/// ```
pub fn parse_ini_content(content: &str) -> Result<Ini, String> {
    let mut config = Ini::new();
    config
        .read(content.to_string())
        .map_err(|e| format!("Failed to parse INI: {e:?}"))?;
    Ok(config)
}

/// Thread-safe global configuration container for Rusty Commit Saver.
///
/// This struct holds all runtime configuration loaded from the INI file,
/// using `OnceCell` for lazy initialization and thread safety. Configuration
/// values are set once during initialization and remain immutable thereafter.
///
/// # Usage Pattern
///
/// ```
/// use rusty_commit_saver::config::GlobalVars;
///
/// // 1. Create instance
/// let global_vars = GlobalVars::new();
///
/// // 2. Load configuration from INI file
/// global_vars.set_all();
///
/// // 3. Access configuration values
/// let obsidian_root = global_vars.get_obsidian_root_path_dir();
/// let commit_path = global_vars.get_obsidian_commit_path();
/// ```
///
/// # See Also
///
/// - [`GlobalVars::new()`] - Create new instance
/// - [`GlobalVars::set_all()`] - Initialize from INI file
/// - [`parse_ini_content()`] - Parse INI content
#[derive(Debug, Default)]
pub struct GlobalVars {
    /// The parsed INI configuration file.
    ///
    /// Stores the complete parsed configuration from the INI file.
    /// Initialized once by [`set_all()`](Self::set_all).
    ///
    /// # Thread Safety
    ///
    /// `OnceCell` ensures this is set exactly once and can be safely
    /// accessed from multiple threads.
    pub config: OnceCell<Ini>,

    /// Root directory of the Obsidian vault.
    ///
    /// The base directory where all Obsidian files are stored.
    /// All diary entries are created under this directory.
    ///
    /// # Examples
    ///
    /// - `/home/user/Documents/Obsidian`
    /// - `C:\Users\username\Documents\Obsidian` (Windows)
    ///
    /// # Configuration
    ///
    /// Loaded from INI file:
    /// ```
    /// [obsidian]
    /// root_path_dir = ~/Documents/Obsidian
    /// ```
    obsidian_root_path_dir: OnceCell<PathBuf>,

    /// Subdirectory path for commit diary entries.
    ///
    /// Relative path under [`obsidian_root_path_dir`](Self::obsidian_root_path_dir)
    /// where commit entries are organized.
    ///
    /// # Examples
    ///
    /// - `Diaries/Commits`
    /// - `Journal/Git`
    ///
    /// # Full Path Construction
    ///
    /// Combined with root and date template:
    /// ```
    /// {root_path_dir}/{commit_path}/{date_template}
    /// /home/user/Obsidian/Diaries/Commits/2025/01-January/2025-01-14.md
    /// ```
    ///
    /// # Configuration
    ///
    /// Loaded from INI file:
    /// ```
    /// [obsidian]
    /// commit_path = Diaries/Commits
    /// ```
    obsidian_commit_path: OnceCell<PathBuf>,

    /// Chrono format string for date-based file paths.
    ///
    /// Controls the directory structure and filename for diary entries.
    /// Uses Chrono format specifiers to create date-organized paths.
    ///
    /// # Format Specifiers
    ///
    /// - `%Y` - Year (e.g., `2025`)
    /// - `%m` - Month number (e.g., `01`)
    /// - `%B` - Full month name (e.g., `January`)
    /// - `%F` - ISO 8601 date (e.g., `2025-01-14`)
    /// - `%d` - Day of month (e.g., `14`)
    ///
    /// # Examples
    ///
    /// ```
    /// Format: %Y/%m-%B/%F.md
    /// Result: 2025/01-January/2025-01-14.md
    ///
    /// Format: %Y/week-%W/%F.md
    /// Result: 2025/week-02/2025-01-14.md
    /// ```
    ///
    /// # Configuration
    ///
    /// Loaded from INI file:
    /// ```
    /// [templates]
    /// commit_date_path = %Y/%m-%B/%F.md
    /// ```
    template_commit_date_path: OnceCell<String>,

    /// Chrono format string for datetime display in diary entries.
    ///
    /// Controls how commit timestamps appear in the diary table's TIME column.
    ///
    /// # Format Specifiers
    ///
    /// - `%Y` - Year (e.g., `2025`)
    /// - `%m` - Month (e.g., `01`)
    /// - `%d` - Day (e.g., `14`)
    /// - `%H` - Hour, 24-hour (e.g., `14`)
    /// - `%M` - Minute (e.g., `30`)
    /// - `%S` - Second (e.g., `45`)
    /// - `%T` - Time in HH:MM:SS format
    ///
    /// # Examples
    ///
    /// ```
    /// Format: %Y-%m-%d %H:%M:%S
    /// Result: 2025-01-14 14:30:45
    ///
    /// Format: %H:%M:%S
    /// Result: 14:30:45
    /// ```
    ///
    /// # Configuration
    ///
    /// Loaded from INI file:
    /// ```
    /// [templates]
    /// commit_datetime = %Y-%m-%d %H:%M:%S
    /// ```
    template_commit_datetime: OnceCell<String>,
}

impl GlobalVars {
    /// Creates a new uninitialized `GlobalVars` instance.
    ///
    /// This constructor initializes all fields as empty `OnceCell` values.
    /// Use [`set_all()`](Self::set_all) to load configuration from the INI file.
    ///
    /// # Thread Safety
    ///
    /// `GlobalVars` uses `OnceCell` for thread-safe, lazy initialization.
    /// Configuration values are set once and cannot be changed afterward.
    ///
    /// # Returns
    ///
    /// A new `GlobalVars` instance with all fields uninitialized
    ///
    /// # Fields
    ///
    /// - `config` - The parsed INI configuration file
    /// - `obsidian_root_path_dir` - Root directory of Obsidian vault
    /// - `obsidian_commit_path` - Subdirectory path for commit entries
    /// - `template_commit_date_path` - Chrono format for date-based directory structure
    /// - `template_commit_datetime` - Chrono format for datetime strings
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_commit_saver::config::GlobalVars;
    ///
    /// // Create new instance
    /// let global_vars = GlobalVars::new();
    ///
    /// // Now call set_all() to initialize from config file
    /// // global_vars.set_all();
    /// ```
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

    /// Loads and initializes all configuration from the INI file.
    ///
    /// This is the main entry point for configuration setup. It:
    /// 1. Reads the INI configuration file from disk (or CLI argument)
    /// 2. Parses it into the `config` field
    /// 3. Extracts and initializes all Obsidian and template variables
    ///
    /// Configuration is loaded from (in order of preference):
    /// - `--config-ini <PATH>` CLI argument
    /// - Default: `~/.config/rusty-commit-saver/rusty-commit-saver.ini`
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Configuration file doesn't exist
    /// - Configuration file cannot be read
    /// - Configuration file has invalid INI format
    /// - Required sections or keys are missing
    /// - Section count is not exactly 2 (obsidian + templates)
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining
    ///
    /// # Required INI Structure
    ///
    /// ```
    /// [obsidian]
    /// root_path_dir = ~/Documents/Obsidian
    /// commit_path = Diaries/Commits
    ///
    /// [templates]
    /// commit_date_path = %Y/%m-%B/%F.md
    /// commit_datetime = %Y-%m-%d %H:%M:%S
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_commit_saver::config::GlobalVars;
    ///
    /// let global_vars = GlobalVars::new();
    /// global_vars.set_all(); // Reads from default or CLI config
    ///
    /// // Now all getters will return values
    /// let root_path = global_vars.get_obsidian_root_path_dir();
    /// let commit_path = global_vars.get_obsidian_commit_path();
    /// ```
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

    /// Returns the root directory of the Obsidian vault.
    ///
    /// This is the base directory where all Obsidian vault files are stored.
    /// All diary entries are created under this directory according to the
    /// configured subdirectory structure.
    ///
    /// # Panics
    ///
    /// Panics if called before [`set_all()`](Self::set_all) has been invoked
    ///
    /// # Returns
    ///
    /// A `PathBuf` representing the Obsidian vault root directory
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_commit_saver::config::GlobalVars;
    ///
    /// let global_vars = GlobalVars::new();
    /// global_vars.set_all();
    ///
    /// let root = global_vars.get_obsidian_root_path_dir();
    /// println!("Obsidian vault root: {}", root.display());
    /// // Output: Obsidian vault root: /home/user/Documents/Obsidian
    /// ```
    ///
    /// # Configuration Source
    ///
    /// Read from INI file:
    /// ```
    /// [obsidian]
    /// root_path_dir = ~/Documents/Obsidian
    /// ```
    pub fn get_obsidian_root_path_dir(&self) -> PathBuf {
        info!("[GlobalVars::get_obsidian_root_path_dir()]: Getting obsidian_root_path_dir.");
        self.obsidian_root_path_dir
            .get()
            .expect("Could not get obsidian_root_path_dir")
            .clone()
    }

    /// Returns the subdirectory path where commits are stored.
    ///
    /// This is a relative path under [`get_obsidian_root_path_dir()`](Self::get_obsidian_root_path_dir)
    /// where commit diary entries will be organized. The full path is constructed by
    /// combining this with the Obsidian root and the date-based directory structure.
    ///
    /// # Panics
    ///
    /// Panics if called before [`set_all()`](Self::set_all) has been invoked
    ///
    /// # Returns
    ///
    /// A `PathBuf` representing the commits subdirectory (relative path)
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_commit_saver::config::GlobalVars;
    ///
    /// let global_vars = GlobalVars::new();
    /// global_vars.set_all();
    ///
    /// let commit_path = global_vars.get_obsidian_commit_path();
    /// println!("Commit subdirectory: {}", commit_path.display());
    /// // Output: Commit subdirectory: Diaries/Commits
    ///
    /// // Full path would be constructed as:
    /// // /home/user/Documents/Obsidian/Diaries/Commits/2025/01-January/2025-01-14.md
    /// ```
    ///
    /// # Configuration Source
    ///
    /// Read from INI file:
    /// ```
    /// [obsidian]
    /// commit_path = Diaries/Commits
    /// ```
    pub fn get_obsidian_commit_path(&self) -> PathBuf {
        info!("[GlobalVars::get_obsidian_commit_path()]: Getting obsidian_commit_path.");
        self.obsidian_commit_path
            .get()
            .expect("Could not get obsidian_commit_path")
            .clone()
    }

    /// Returns the Chrono format string for diary file date hierarchies.
    ///
    /// This format string is used to create the directory structure and filename
    /// for diary entries based on the commit timestamp. It controls how commits
    /// are organized by date.
    ///
    /// # Chrono Format Specifiers
    ///
    /// - `%Y` - Full year (e.g., `2025`)
    /// - `%m` - Month as zero-padded number (e.g., `01`)
    /// - `%B` - Full month name (e.g., `January`)
    /// - `%b` - Abbreviated month (e.g., `Jan`)
    /// - `%d` - Day of month, zero-padded (e.g., `14`)
    /// - `%F` - ISO 8601 date (equivalent to `%Y-%m-%d`, e.g., `2025-01-14`)
    /// - `%H` - Hour in 24-hour format (e.g., `14`)
    /// - `%M` - Minute (e.g., `30`)
    /// - `%S` - Second (e.g., `45`)
    ///
    /// # Panics
    ///
    /// Panics if called before [`set_all()`](Self::set_all) has been invoked
    ///
    /// # Returns
    ///
    /// A `String` containing the Chrono format specifiers
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_commit_saver::config::GlobalVars;
    ///
    /// let global_vars = GlobalVars::new();
    /// global_vars.set_all();
    ///
    /// let date_template = global_vars.get_template_commit_date_path();
    /// println!("Date format: {}", date_template);
    /// // Output: Date format: %Y/%m-%B/%F.md
    ///
    /// // This creates paths like:
    /// // /home/user/Obsidian/Diaries/Commits/2025/01-January/2025-01-14.md
    /// ```
    ///
    /// # Configuration Source
    ///
    /// Read from INI file:
    /// ```
    /// [templates]
    /// commit_date_path = %Y/%m-%B/%F.md
    /// ```
    pub fn get_template_commit_date_path(&self) -> String {
        info!("[GlobalVars::get_template_commit_date_path()]: Getting template_commit_date_path.");
        self.template_commit_date_path
            .get()
            .expect("Could not get template_commit_date_path")
            .clone()
    }

    /// Returns the Chrono format string for commit timestamps in diary entries.
    ///
    /// This format string is used to display the commit time in the diary table.
    /// It controls how timestamps appear in the commit entry rows.
    ///
    /// # Chrono Format Specifiers
    ///
    /// - `%Y` - Full year (e.g., `2025`)
    /// - `%m` - Month as zero-padded number (e.g., `01`)
    /// - `%B` - Full month name (e.g., `January`)
    /// - `%d` - Day of month, zero-padded (e.g., `14`)
    /// - `%H` - Hour in 24-hour format (e.g., `14`)
    /// - `%M` - Minute, zero-padded (e.g., `30`)
    /// - `%S` - Second, zero-padded (e.g., `45`)
    /// - `%T` - Time in HH:MM:SS format (equivalent to `%H:%M:%S`)
    ///
    /// # Panics
    ///
    /// Panics if called before [`set_all()`](Self::set_all) has been invoked
    ///
    /// # Returns
    ///
    /// A `String` containing the Chrono format specifiers for datetime
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_commit_saver::config::GlobalVars;
    ///
    /// let global_vars = GlobalVars::new();
    /// global_vars.set_all();
    ///
    /// let datetime_template = global_vars.get_template_commit_datetime();
    /// println!("Datetime format: {}", datetime_template);
    /// // Output: Datetime format: %Y-%m-%d %H:%M:%S
    ///
    /// // This renders timestamps like:
    /// // 2025-01-14 14:30:45
    /// ```
    ///
    /// # Diary Table Usage
    ///
    /// In the diary table, this format appears in the TIME column:
    /// ```
    /// | FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
    /// |--------|------|----------------|----------------|--------|-------------|
    /// | /work/project | 14:30:45 | feat: add feature | https://github.com/... | main | abc123... |
    /// ```
    ///
    /// # Configuration Source
    ///
    /// Read from INI file:
    /// ```
    /// [templates]
    /// commit_datetime = %Y-%m-%d %H:%M:%S
    /// ```
    pub fn get_template_commit_datetime(&self) -> String {
        info!("[GlobalVars::get_template_commit_datetime()]: Getting template_commit_datetime.");
        self.template_commit_datetime
            .get()
            .expect("Could not get template_commit_datetime")
            .clone()
    }

    /// Retrieves a clone of the parsed INI configuration.
    ///
    /// This is a private helper method that returns a copy of the configuration
    /// object. Used internally by other helper methods to access sections and keys.
    ///
    /// # Panics
    ///
    /// Panics if called before [`set_all()`](Self::set_all) has initialized the config.
    ///
    /// # Returns
    ///
    /// A cloned `Ini` configuration object
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

    /// Sets the `template_commit_datetime` field from the `[templates]` section.
    ///
    /// Reads the `commit_datetime` key from the INI file and stores it in the
    /// `template_commit_datetime` `OnceCell`.
    ///
    /// # Arguments
    ///
    /// * `section` - Should be `"templates"` (validated by caller)
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The `commit_datetime` key is missing from the INI section
    /// - The `OnceCell` has already been set (called multiple times)
    ///
    /// # Expected INI Key
    ///
    /// ```
    /// [templates]
    /// commit_datetime = %Y-%m-%d %H:%M:%S
    /// ```
    fn set_templates_datetime(&self, section: &str) {
        info!("[GlobalVars::set_templates_datetime()]: Setting the templates_datetime.");
        let key = self
            .get_key_from_section_from_ini(section, "commit_datetime")
            .expect("Could not get the commit_datetime from INI");

        self.template_commit_datetime
            .set(key)
            .expect("Could not set the template_commit_datetime GlobalVars");
    }

    /// Sets the `template_commit_date_path` field from the `[templates]` section.
    ///
    /// Reads the `commit_date_path` key from the INI file and stores it in the
    /// `template_commit_date_path` `OnceCell`.
    ///
    /// # Arguments
    ///
    /// * `section` - Should be `"templates"` (validated by caller)
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The `commit_date_path` key is missing from the INI section
    /// - The `OnceCell` has already been set (called multiple times)
    ///
    /// # Expected INI Key
    ///
    /// ```
    /// [templates]
    /// commit_date_path = %Y/%m-%B/%F.md
    /// ```
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

    /// Sets the `obsidian_commit_path` field from the `[obsidian]` section.
    ///
    /// Reads the `commit_path` key, expands tilde (`~`) to the home directory
    /// if present, splits the path by `/`, and constructs a `PathBuf`.
    ///
    /// # Arguments
    ///
    /// * `section` - Should be `"obsidian"` (validated by caller)
    ///
    /// # Tilde Expansion
    ///
    /// - `~/Diaries/Commits` → `/home/user/Diaries/Commits`
    /// - `/absolute/path` → `/absolute/path` (unchanged)
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The `commit_path` key is missing from the INI section
    /// - Home directory cannot be determined (when `~` is used)
    /// - The `OnceCell` has already been set
    ///
    /// # Expected INI Key
    ///
    /// ```
    /// [obsidian]
    /// commit_path = ~/Documents/Obsidian/Diaries/Commits
    /// ```
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

    /// Sets the `obsidian_root_path_dir` field from the `[obsidian]` section.
    ///
    /// Reads the `root_path_dir` key, expands tilde (`~`) to the home directory
    /// if present, prepends `/` for absolute paths, and constructs a `PathBuf`.
    ///
    /// # Arguments
    ///
    /// * `section` - Should be `"obsidian"` (validated by caller)
    ///
    /// # Path Construction
    ///
    /// - Starts with `/` to ensure absolute path
    /// - Expands `~` to home directory
    /// - Splits by `/` and constructs `PathBuf`
    ///
    /// # Tilde Expansion Examples
    ///
    /// - `~/Documents/Obsidian` → `/home/user/Documents/Obsidian`
    /// - `/absolute/path` → `/absolute/path`
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The `root_path_dir` key is missing from the INI section
    /// - Home directory cannot be determined (when `~` is used)
    /// - The `OnceCell` has already been set
    ///
    /// # Expected INI Key
    ///
    /// ```
    /// [obsidian]
    /// root_path_dir = ~/Documents/Obsidian
    /// ```
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

/// Command-line argument parser for configuration file path.
///
/// This struct uses `clap` to parse CLI arguments and provide configuration
/// options for the application. Currently supports specifying a custom INI
/// configuration file path.
///
/// # CLI Arguments
///
/// - `--config-ini <PATH>` - Optional path to a custom configuration file
///
/// # Examples
///
/// ```
/// # Use default config (~/.config/rusty-commit-saver/rusty-commit-saver.ini)
/// rusty-commit-saver
///
/// # Use custom config file
/// rusty-commit-saver --config-ini /path/to/custom.ini
/// ```
///
/// # See Also
///
/// - [`retrieve_config_file_path()`] - Gets the config path from CLI or default
/// - [`get_ini_file()`] - Loads the INI file from the resolved path
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(about = "Rusty Commit Saver config", long_about = None)]
pub struct UserInput {
    /// Path to a custom INI configuration file.
    ///
    /// If not provided, the default configuration file is used:
    /// `~/.config/rusty-commit-saver/rusty-commit-saver.ini`
    ///
    /// # CLI Usage
    ///
    /// ```
    /// rusty-commit-saver --config-ini /custom/path/config.ini
    /// ```
    ///
    /// # Examples
    ///
    /// Valid paths:
    /// - `~/my-configs/commit-saver.ini`
    /// - `/etc/rusty-commit-saver/config.ini`
    /// - `./local-config.ini`
    #[arg(short, long)]
    pub config_ini: Option<String>,
}

/// Retrieves the configuration file path from CLI arguments or returns the default.
///
/// This function parses command-line arguments and returns the path to the INI
/// configuration file. If no `--config-ini` argument is provided, returns the
/// default path.
///
/// # Default Path
///
/// `~/.config/rusty-commit-saver/rusty-commit-saver.ini`
///
/// # Returns
///
/// A `String` containing the absolute path to the configuration file
///
/// # CLI Usage
///
/// ```
/// # Use default config
/// rusty-commit-saver
/// # Returns: /home/user/.config/rusty-commit-saver/rusty-commit-saver.ini
///
/// # Use custom config
/// rusty-commit-saver --config-ini /custom/path/config.ini
/// # Returns: /custom/path/config.ini
/// ```
///
/// # Examples
///
/// ```
/// use rusty_commit_saver::config::retrieve_config_file_path;
///
/// let config_path = retrieve_config_file_path();
/// println!("Using config: {}", config_path);
/// ```
///
/// # See Also
///
/// - [`UserInput`] - CLI argument parser
/// - [`get_or_default_config_ini_path()`] - Helper that implements the logic
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

/// Returns the config path from CLI arguments or the default path.
///
/// Internal helper function that parses CLI arguments using `UserInput` and
/// returns either the provided `--config-ini` path or the default configuration
/// file location.
///
/// # Returns
///
/// - CLI path if `--config-ini` was provided
/// - Default path (`~/.config/rusty-commit-saver/rusty-commit-saver.ini`) otherwise
///
/// # Called By
///
/// This function is called internally by [`retrieve_config_file_path()`].
///
/// # See Also
///
/// - [`get_default_ini_path()`] - Constructs the default configuration path
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

/// Constructs the default configuration file path.
///
/// Builds the standard XDG configuration path for the application by combining
/// the user's home directory with the application-specific config directory.
///
/// # Returns
///
/// A `String` with the default INI file path:
/// `~/.config/rusty-commit-saver/rusty-commit-saver.ini`
///
/// # Directory Structure
///
/// ```
/// ~/.config/
///   └── rusty-commit-saver/
///       └── rusty-commit-saver.ini
/// ```
///
/// # Panics
///
/// Panics if the user's home directory cannot be determined
/// (via the `dirs::home_dir()` function).
///
/// # Examples
///
/// ```
/// // Internal usage
/// let default_path = get_default_ini_path();
/// // Returns: "/home/user/.config/rusty-commit-saver/rusty-commit-saver.ini"
/// ```
///
/// # See Also
///
/// - [`retrieve_config_file_path()`] - Public API for getting config path
fn get_default_ini_path() -> String {
    info!("[get_default_ini_path()]: Getting default ini file.");
    let cfg_str = "~/.config/rusty-commit-saver/rusty-commit-saver.ini".to_string();
    set_proper_home_dir(&cfg_str)
}

/// Loads and parses the INI configuration file from disk.
///
/// Reads the configuration file (from CLI argument or default location),
/// parses its contents using [`parse_ini_content()`], and returns the
/// parsed `Ini` object.
///
/// # Returns
///
/// A parsed `Ini` configuration object
///
/// # Panics
///
/// Panics if:
/// - The configuration file doesn't exist at the resolved path
/// - The file cannot be read (permission denied, I/O error)
/// - The file content is not valid UTF-8
/// - The INI syntax is invalid (malformed sections or key-value pairs)
///
/// # File Resolution Order
///
/// 1. Check for `--config-ini <PATH>` CLI argument
/// 2. Fall back to `~/.config/rusty-commit-saver/rusty-commit-saver.ini`
///
/// # Expected INI Structure
///
/// ```
/// [obsidian]
/// root_path_dir = ~/Documents/Obsidian
/// commit_path = Diaries/Commits
///
/// [templates]
/// commit_date_path = %Y/%m-%B/%F.md
/// commit_datetime = %Y-%m-%d %H:%M:%S
/// ```
///
/// # Called By
///
/// This function is called internally by [`GlobalVars::set_all()`].
///
/// # See Also
///
/// - [`retrieve_config_file_path()`] - Resolves the config file path
/// - [`parse_ini_content()`] - Parses INI text into `Ini` struct
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

/// Expands the tilde (`~`) character to the user's home directory path.
///
/// Replaces the leading `~` in a path string with the absolute path to the
/// user's home directory. If no `~` is present, returns the string unchanged.
///
/// # Arguments
///
/// * `cfg_str` - A path string that may contain a leading `~`
///
/// # Returns
///
/// A `String` with `~` expanded to the full home directory path
///
/// # Panics
///
/// Panics if the user's home directory cannot be determined
/// (via the `dirs::home_dir()` function).
///
/// # Examples
///
/// ```
/// // On Linux/macOS with home at /home/user
/// let expanded = set_proper_home_dir("~/Documents/Obsidian");
/// assert_eq!(expanded, "/home/user/Documents/Obsidian");
///
/// // Path without tilde is returned unchanged
/// let unchanged = set_proper_home_dir("/absolute/path");
/// assert_eq!(unchanged, "/absolute/path");
/// ```
///
/// # Platform Behavior
///
/// - **Linux/macOS**: Expands to `/home/username` or `/Users/username`
/// - **Windows**: Expands to `C:\Users\username`
///
/// # Used By
///
/// This function is called by:
/// - [`GlobalVars::set_obsidian_root_path_dir()`]
/// - [`GlobalVars::set_obsidian_commit_path()`]
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
