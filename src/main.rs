///!
///! Git commit save to VimWiki
///!

/// Standard Lib
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// External crates
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use dirs::home_dir;
use git2::Repository;

#[derive(Debug, Clone)]
struct VimCommit {
    pub repository_url: String,
    pub branch_name: String,
    pub commit_hash: String,
    pub commit_msg: String,
    pub datetime: String,
}

/// Defaults for VimCommit
impl Default for VimCommit {
    fn default() -> VimCommit {
        VimCommit {
            repository_url: {
                let git_repo = Repository::discover("./").unwrap();
                let bind = git_repo.find_remote("origin").unwrap();
                bind.url().unwrap().replace('\"', "")
            },
            branch_name: {
                let git_repo = Repository::discover("./").unwrap();
                let head = git_repo.head().unwrap();
                head.shorthand().unwrap().replace('\"', "")
            },
            commit_hash: {
                let git_repo = Repository::discover("./").unwrap();
                let head = git_repo.head().unwrap();
                let cbind = head.peel_to_commit().unwrap();
                cbind.id().to_string()
            },
            commit_msg: {
                let git_repo = Repository::discover("./").unwrap();
                let head = git_repo.head().unwrap();
                let cbind = head.peel_to_commit().unwrap();
                cbind.message().unwrap().replace(['\n', '\"'], "")
            },
            datetime: {
                let git_repo = Repository::discover("./").unwrap();
                let head = git_repo.head().unwrap();

                let cbind = head.peel_to_commit().unwrap();
                let commit_date: i64 = cbind.time().seconds();
                let dt: DateTime<Utc> = DateTime::from_utc(
                    NaiveDateTime::from_timestamp_opt(commit_date, 0).unwrap(),
                    Utc,
                );
                dt.format("%H:%M:%S").to_string()
            },
        }
    }
}

impl VimCommit {
    pub fn new() -> Self {
        Default::default()
    }

    /// Prepares input to write to vimwiki
    fn prepare_input(&mut self) -> String {
        format!(
            "| {:} | {:} | {:} | {:} | {:} |\n",
            self.datetime, self.commit_msg, self.repository_url, self.branch_name, self.commit_hash
        )
    }

    /// Method to veritfy that the file exists
    /// will trigger the creation of it with a template if it doesn't
    fn check_diary_day_exists(&mut self, vimwiki: &str) -> PathBuf {
        let today: DateTime<Utc> = Utc::now();
        let md_file = format!("{}.md", today.format("%Y-%m-%d"));
        let mut wikidir = home_dir().unwrap();
        wikidir.push(&[vimwiki, "diary/", &md_file].iter().collect::<PathBuf>());
        if !Path::new(&wikidir).exists() {
            println!(
                "Diary entry doesn't exist, we're now creating it: {:?}",
                wikidir.as_os_str().to_str().unwrap()
            );
            self.mkfile(&wikidir);
        }
        wikidir
    }

    /// Create file
    fn mkfile(&mut self, wikidir: &PathBuf) {
        let today: DateTime<Utc> = Utc::now();
        let md_title = if wikidir
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string()
            .contains("transics")
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
## Personal projects notes:\n\n 
## Todays' commits:\n
| TIME | COMMIT MESSAGE | REPOSITORY URL | BRANCH | COMMIT HASH |
|------|----------------|----------------|--------|-------------|
",
                today.format("%Y-%m-%d")
            )
        };

        fs::write(wikidir, md_title)
            .expect("Something went wront creating diary file and writing things in it");
    }

    /// Append git stuff to diary
    fn append_commit_stuff_to_diary(&mut self, wiki: &PathBuf) {
        let new_commit_str = self.prepare_input();

        let mut file_ref = OpenOptions::new()
            .append(true)
            .open(wiki)
            .expect("Unable to open wiki");

        file_ref
            .write_all(new_commit_str.as_bytes())
            .expect("Failed to write the new commit string");

        println!("Wrote new commit to your logbook!");
    }

    /// check if remote is for transics or else.
    fn select_proper_diary(&mut self) -> String {
        if self.repository_url.contains("transics") {
            return ".vimwiki/work/".to_string();
        }
        ".vimwiki/home/".to_string()
    }
}

fn main() {
    let mut vimc = VimCommit::new();

    let wikifile = vimc.select_proper_diary();
    let wiki = vimc.check_diary_day_exists(&wikifile);
    vimc.append_commit_stuff_to_diary(&wiki);
}
