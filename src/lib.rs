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
//! ```ignore
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
//! ```text
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
