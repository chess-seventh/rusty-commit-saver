//!
//! Save all my commits to Obisidian
//!

pub mod vim_commit;
use vim_commit::CommitSaver;

use dirs::home_dir;
use log::error;
use log::info;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

markup::define! {
    DiaryFileEntry(frontmatter: Vec<String>, diary_date: String) {
"---
category: diary\n
section: commits\n
tags:\n"
@for tag in frontmatter.iter() {
"- '" @tag "'\n"
}
"date: " @diary_date
"\n
---
\n
# " @diary_date
"\n
| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
|--------|------|----------------|----------------|--------|-------------|\n"
    }
}

fn get_parent_from_full_path(full_diary_path: &Path) -> Result<&Path, Box<dyn Error>> {
    match full_diary_path.parent() {
        Some(dir) => Ok(dir),
        None => Err("Something went wrong when getting the parent directory".into()),
    }
}

/// Method to veritfy that the file exists
/// Will trigger the creation of it with a template if it doesn't
fn check_diary_path_exists(full_diary_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if Path::new(&full_diary_path).exists() {
        return Ok(());
    }
    Err("Path does not exist!".into())
}

fn create_diary_file(
    full_diary_file_path: &str,
    commit_saver_struct: &mut CommitSaver,
) -> Result<(), Box<dyn Error>> {
    let frontmatter = commit_saver_struct.prepare_frontmatter_tags();
    let diary_date = commit_saver_struct
        .commit_datetime
        .format("%Y-%m-%d")
        .to_string();

    let template = DiaryFileEntry {
        frontmatter,
        diary_date,
    }
    .to_string();
    fs::write(full_diary_file_path, template)?;

    Ok(())
}

fn create_directories_for_new_entry(entry_directory_and_path: &Path) -> Result<(), Box<dyn Error>> {
    let parent_dirs = get_parent_from_full_path(entry_directory_and_path)?;
    fs::create_dir_all(parent_dirs)?;
    info!("[INFO] Creating diary file & path ...........................");
    println!("[INFO] Creating diary file & path ...........................");

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut commit_saver_struct = CommitSaver::new();

    let current_directory = env::current_dir().unwrap();
    let mut entry_directory_and_path = home_dir().unwrap();
    entry_directory_and_path.push("Documents");
    entry_directory_and_path.push("Wiki");

    if current_directory == entry_directory_and_path {
        info!("[INFO] No need to save the commit here ......................");
        println!("[INFO] No need to save the commit here ......................");
        return Ok(());
    }

    let diary_entry_path = commit_saver_struct.prepare_path_for_commit();
    entry_directory_and_path.push(diary_entry_path);

    let tmp = &entry_directory_and_path.as_os_str().to_str().unwrap();

    if let Ok(()) = check_diary_path_exists(&entry_directory_and_path) {
        info!("[INFO] Diary file/path exists ...............................");
        info!("[INFO] {tmp:} ...............................");
        println!("[INFO] Diary file/path exists ...............................");
        println!("[INFO] {tmp:} ...............................");
    } else {
        info!("[INFO] Diary file/path DOES NOT exist .......................");
        println!("[INFO] Diary file/path DOES NOT exist .......................");
        create_directories_for_new_entry(&entry_directory_and_path)?;
        create_diary_file(
            entry_directory_and_path.as_os_str().to_str().unwrap(),
            &mut commit_saver_struct,
        )?;
    }

    // write commit
    match commit_saver_struct.append_entry_to_diary(&entry_directory_and_path) {
        Ok(()) => {
            info!("[INFO] Commit logged in .....................................");
            println!("[INFO] Commit logged in .....................................");
            Ok(())
        }
        Err(e) => {
            error!("[ERROR] {e:}");
            println!("[ERROR] {e:}");
            panic!("Something went wrong when writing the commit to the file");
        }
    }
}
