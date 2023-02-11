use chrono::DateTime;
use chrono::Utc;
use dirs::home_dir;
use std::fmt;
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
// use std::process::Command;

use git2::Repository;

struct VimCommit {
    pub repository_url: String,
    pub branch_name: String,
    pub commit_hash: String,
    pub commit_msg: String,
    pub date: DateTime<Utc>,
}

// impl Debug for VimCommit {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!("[{self.repository_url:?}] on {self.branch_name:?} - [{self.commit_id:?}] : {self.commit_msg:?}");
//     }
// }

fn prepare_input() -> String {
    let git_repo = Repository::discover("./").unwrap();
    let head = git_repo.head().unwrap();

    let cbind = head.peel_to_commit().unwrap();
    let commit_id = cbind.id();
    let commit_msg = cbind.message().unwrap();

    let branch_name = head.shorthand().unwrap().replace("\"", "");

    let bind = git_repo.find_remote("origin").unwrap();
    let remote = bind.url().unwrap().replace("\"", "");

    let vim_input = format!("[{remote:?}] on {branch_name:?} - [{commit_id:?}] : {commit_msg:?}");

    vim_input
}

/// when running hook
fn check_diary_day_exists(vimwiki: &str) -> Option<PathBuf> {
    let today: DateTime<Utc> = Utc::now();
    let md_file = format!("{}.md", today.format("%Y-%m-%d"));
    let mut wikidir = home_dir().unwrap();
    wikidir.push(&[vimwiki, "diary/", &md_file].iter().collect::<PathBuf>());
    if !Path::new(&wikidir).exists() {
        mkfile(&wikidir);
    }
    None
}

/// Create file
fn mkfile(wikidir: &PathBuf) {
    let today: DateTime<Utc> = Utc::now();
    let md_title = format!("# {}", today.format("%Y-%m-%d"));

    fs::write(&wikidir, &md_title).expect("Something went wront creating file and writing");
}

/// Append git stuff to diary
fn append_commit_stuff_to_diary() {
    todo!()
}

/// check if remote is for transics or else.
fn select_proper_diary() -> String {
    let git_repo = Repository::discover("./").unwrap();
    let bind = git_repo.find_remote("origin").unwrap();
    let remote = bind.url().unwrap().replace("\"", "");
    if remote.contains("transics") {
        return ".vimwikiwork/".to_string();
    }
    ".vimwiki/".to_string()
}

fn main() {
    let vim_input = prepare_input();
    println!("{vim_input:}");
    // let cmd = Command::new("vim")
}
