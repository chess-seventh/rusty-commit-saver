//!
//! Save all you commits to your VimWiki
//!

pub mod vim_commit;
use vim_commit::CommitSaver;

pub mod git_repository;

use log::info;
use std::env;
use std::error::Error;
use std::fs;
use std::iter::Zip;
use std::path::Path;
use std::path::PathBuf;

use dirs::home_dir;

use markup;

fn prepare_template_for_new_diary_file(
    frontmatter: Vec<String>,
    diary_date: String,
    diary_path: String,
    commit_entry: String,
) {
    markup::define! {
        DiaryFileEntry(frontmatter: Vec<String>, diary_date: String, diary_path: String, commit_entry: String) {
        "---
            \n
            category: diary\n
            section: home\n
            tags:\n"
        @for tag in frontmatter.iter() {
          "-" @tag
        }
        "\ndate: \"" diary_date "\""
            "\n
            ---
            \n
            \n
            # ["diary_date"]("diary_path")\n\n
            | FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
            |--------|------|----------------|----------------|--------|-------------|\n"
            commit_entry;
        }
    }
}

fn get_parent_from_full_path(full_diary_path: &PathBuf) -> Result<&Path, Box<dyn Error>> {
    match full_diary_path.parent() {
        Some(dir) => Ok(dir),
        None => Err("Something went wrong when getting the parent directory".into()),
    }
}

/// Method to veritfy that the file exists
/// Will trigger the creation of it with a template if it doesn't
fn check_diary_path_exists(full_diary_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let parent_dirs = get_parent_from_full_path(full_diary_path)?;

    // Recursively create a directory and all of its parent components if they are missing.
    // https://stackoverflow.com/a/48053959
    if !Path::new(&parent_dirs).exists() {
        fs::create_dir_all(parent_dirs)?;
        info!("[INFO] Diary file does not exist ............................");
        info!(
            "[INFO] Creating Diary file {:?} .............................",
            full_diary_path.as_os_str().to_str().unwrap()
        );

        let frontmatter = vec!["hello".to_string(), "world".to_string()];

        let template =
            prepare_template_for_new_diary_file(frontmatter, diary_date, diary_path, commit_entry);

        // let _template = markup::new! {
        //     "---
        //     \n
        //     category: diary\n
        //     section: home\n
        //     tags:\n"
        //     @for tag in frontmatter.iter() {
        //       "-" @tag
        //     }
        //     "\n
        //     date: \"@diary_date\"
        //     \n
        //     ---
        //     \n
        //     \n
        //     # [@diary_date](@diary_path)
        //     \n
        //     \n
        //     | FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
        //     |--------|------|----------------|----------------|--------|-------------|\n"
        // };
    }
    Ok(())
}

fn create_diary_file(full_diary_file_path: &str) -> Result<(), Box<dyn Error>> {}

fn main() {
    let mut vimc = CommitSaver::new();

    let current_directory = env::current_dir().unwrap();
    let mut entry_directory_and_path = home_dir().unwrap();
    entry_directory_and_path.push(".vimwiki");

    if current_directory == entry_directory_and_path {
        info!("[INFO] No need to save the wikidir commits ................");
        return;
    }

    let diary_entry_path = vimc.prepare_path_for_commit();
    entry_directory_and_path.push(diary_entry_path);

    match check_diary_path_exists(&entry_directory_and_path) {
        Ok(()) => info!("Diary path created or existed"),
        Err(e) => panic!("Something went wrong with the creation of diary path: {e:}"),
    };

    vimc.append_entry_to_diary(&wiki);
}
