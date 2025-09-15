//! Rusty Commit Saver Library
//!
//! This library provides functionality to save git commits to Obsidian diary entries.

pub mod config;
pub mod vim_commit;

pub use config::{GlobalVars, UserInput};
pub use vim_commit::CommitSaver;
