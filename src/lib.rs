//! # Rusty Commit Saver
//!
//! A Rust tool to automatically log Git commits into Obsidian diary entries.
//!
//! ## Overview
//!
//! Rusty Commit Saver captures each commit's metadata and appends it to a dated
//! diary entry in your Obsidian vault. Each entry includes:
//!
//! - Timestamp
//! - Commit message
//! - Repository URL
//! - Branch name
//! - Commit hash
//!
//! ## Quick Start
//!
//! ```
//! use rusty_commit_saver::{run_commit_saver, config::GlobalVars};
//! use std::path::PathBuf;
//!
//! // Initialize configuration
//! let global_vars = GlobalVars::new();
//! global_vars.set_all();
//!
//! // Get configuration values
//! let obsidian_root = global_vars.get_obsidian_root_path_dir();
//! let commit_path = global_vars.get_obsidian_commit_path();
//! let date_template = global_vars.get_template_commit_date_path();
//!
//! // Save the commit
//! run_commit_saver(obsidian_root, &commit_path, &date_template).unwrap();
//! ```
//!
//! ## Configuration
//!
//! Configuration is stored in an INI file at:
//! `~/.config/rusty-commit-saver/rusty-commit-saver.ini`
//!
//! Example configuration:
//!
//! ```
//! [obsidian]
//! root_path_dir = ~/Documents/Obsidian
//! commit_path = Diaries/Commits
//!
//! [templates]
//! commit_date_path = %Y/%m-%B/%F.md
//! commit_datetime = %Y-%m-%d %H:%M:%S
//! ```
//!
//! ## Modules
//!
//! - [`vim_commit`] - Core commit processing and diary file operations
//! - [`config`] - Configuration management and INI file parsing
//!
//! ## Features
//!
//! - ✅ Automatic diary entry creation with YAML frontmatter
//! - ✅ Timestamped commit rows formatted for Obsidian
//! - ✅ Customizable storage path with date-based organization
//! - ✅ Pipe escaping in commit messages for Markdown table safety
//! - ✅ Thread-safe configuration with `OnceCell`

pub mod config;
pub mod vim_commit;

use log::info;
use std::error::Error;
use std::path::{Path, PathBuf};
use vim_commit::{
    CommitSaver, check_diary_path_exists, create_diary_file, create_directories_for_new_entry,
};

/// Core logic for saving a commit to an Obsidian diary file.
///
/// (Full documentation from Function 9 above)
pub fn run_commit_saver(
    obsidian_root_path_dir: PathBuf,
    obsidian_commit_path: &Path,
    template_commit_date_path: &str,
) -> Result<(), Box<dyn Error>> {
    info!("[run_commit_saver()]: Instanciating CommitSaver Struct");
    let mut commit_saver_struct = CommitSaver::new();

    info!("[run_commit_saver()]: Preparing the diary entry path to the new commit.");
    let diary_entry_path = commit_saver_struct
        .prepare_path_for_commit(obsidian_commit_path, template_commit_date_path);

    let mut full_path = obsidian_root_path_dir;
    for directory in diary_entry_path.split('/') {
        full_path.push(directory);
    }

    let stringed_root_path_dir = full_path
        .as_os_str()
        .to_str()
        .ok_or("Could not convert path to string")?;

    info!("[run_commit_saver()]: Checking if Diary file and/or path exists.");
    if check_diary_path_exists(&full_path).is_ok() {
        info!("[run_commit_saver()]: Diary file and path exists: {stringed_root_path_dir:}");
    } else {
        info!("[run_commit_saver()]: Diary file and or path DO NOT exist.");
        info!("[run_commit_saver()]: Creating the directories for the new entry.");
        create_directories_for_new_entry(&full_path)?;

        info!("[run_commit_saver()]: Creating the files for the new entry.");
        create_diary_file(stringed_root_path_dir, &mut commit_saver_struct)?;
    }

    info!("[run_commit_saver()]: Writing the commit in the file.");
    commit_saver_struct.append_entry_to_diary(&full_path)?;
    info!("[run_commit_saver]: Commit logged in ");

    Ok(())
}
