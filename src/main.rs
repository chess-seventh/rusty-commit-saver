//!
//! Save all you commits to your VimWiki
//!

pub mod vim_commit;
use vim_commit::CommitSaver;

/// Standard Lib
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// External crates
use chrono::DateTime;
use chrono::Utc;
use dirs::home_dir;

/// Create file
fn mkfile(wikidir: &PathBuf) {
    let today: DateTime<Utc> = Utc::now();
    let md_title = if wikidir
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
        .contains("Work")
    {
        format!(
            "# {}\n\n
## AM\n\n
## PM\n\n
## Todays' commits:\n
| TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
|------|----------------|----------------|--------|-------------|
",
            today.format("%Y-%m-%d")
        )
    } else {
        format!(
            "# {}\n\n
## Food:\n\n
- breakfast:\n
- lunch:\n
- dinner:\n\n
## Personal notes:\n\n
## Todays' commits:\n
| TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
|------|----------------|----------------|--------|-------------|
",
            today.format("%Y-%m-%d")
        )
    };

    fs::write(wikidir, md_title)
        .expect("Something went wrong creating in writing things in the wikifile");
}

/// Method to veritfy that the file exists
/// will trigger the creation of it with a template if it doesn't
fn check_diary_day_exists(vimwiki: &str) -> PathBuf {
    let today: DateTime<Utc> = Utc::now();
    let md_file = format!("{}.md", today.format("%Y-%m-%d"));
    let mut wikidir = home_dir().unwrap();

    // Recursively create a directory and all of its parent components if they are missing.
    // https://stackoverflow.com/a/48053959
    fs::create_dir_all(wikidir.clone().into_os_string().to_str().unwrap())
        .expect("Couldn't create the directory");

    wikidir.push(&[&vimwiki, &md_file.as_str()].iter().collect::<PathBuf>());

    if !Path::new(&wikidir).exists() {
        println!(
            "Diary entry doesn't exist, we're now creating it: {:?}",
            wikidir.as_os_str().to_str().unwrap()
        );
        mkfile(&wikidir);
    }
    wikidir
}

fn main() {
    let mut vimc = CommitSaver::new();

    let cur_dir = env::current_dir().unwrap();
    let mut wikidir = home_dir().unwrap();
    wikidir.push(".vimwiki");
    if cur_dir == wikidir {
        println!("No need to save the wikidir commits, exiting...");
        return;
    }

    let wikifile = vimc.select_proper_diary_folder();
    let wiki = check_diary_day_exists(&wikifile);
    vimc.append_commit_stuff_to_diary(&wiki);
}
