// use chrono::DateTime;
// use chrono::NaiveDateTime;
// use chrono::Utc;
// use git2::Commit;
// use git2::Error;
// use git2::Reference;
// use git2::Repository;
// // use std::result::Error;
//
// pub fn get_git_repository() -> Repository {
//     match Repository::open("./") {
//         Ok(res) => res,
//         Err(_) => panic!("Could not get a git repository from current directory!"),
//     }
// }
//
// pub struct GitRepository<'a> {
//     // pub git_repository: Repository,
//     pub repository_head: &'a Reference<'a>,
//     pub repository_url: String,
//     pub git_commit: CreatedCommit,
// }
//
// impl GitRepository<'_> {
//     pub fn new() -> Self {
//         GitRepository {
//             repository_url: GitRepository::get_repository_url().unwrap(),
//             repository_head: &GitRepository::get_repository_head().unwrap(),
//             git_commit: GitRepository::get_git_commit_struct(),
//         }
//     }
//
//     // TODO: error handling
//     fn get_repository_head() -> Result<Reference<'static>, git2::Error> {
//         let git_repository: Repository = Repository::open("./")?;
//         git_repository.head().
//     }
//
//     // TODO: error handling
//     fn get_repository_url() -> Result<String, git2::Error> {
//         let git_repository: Repository = Repository::open("./")?;
//         let origin = &git_repository.find_remote("origin")?;
//         // .expect("Should be able to retrieve the Origin");
//         match origin.url() {
//             Some(res) => Ok(res.replace('\"', "")),
//             None => Err(Error::new(
//                 git2::ErrorCode::NotFound,
//                 git2::ErrorClass::None,
//                 "Could not get the git repository URL",
//             )),
//         }
//     }
//
//     fn get_git_commit_struct() -> CreatedCommit {
//         return CreatedCommit::new();
//     }
// }
//
// #[derive(Default)]
// pub struct CreatedCommit {
//     pub commit_branch_name: String,
//     pub commit_hash: String,
//     pub commit_msg: String,
//     pub commit_datetime: DateTime<Utc>,
// }
//
// impl CreatedCommit {
//     pub fn new() -> Self {
//         return CreatedCommit::default();
//     }
//
//     // TODO: error handling
//     // TODO: check input parameter type
//     pub fn set_branch_name(&mut self, git_repository_head: Reference<'static>) {
//         self.commit_branch_name = git_repository_head
//             .shorthand()
//             .expect("Should be able to get the commits branch name")
//             .replace('\"', "")
//     }
//
//     // TODO: error handling
//     // TODO: check input parameter type
//     pub fn set_commit_hash(&mut self, git_repository_head: Reference<'static>) {
//         self.commit_hash = git_repository_head
//             .peel_to_commit()
//             .expect("Should be able to retrieve the commit object")
//             .id()
//             .to_string()
//     }
//
//     // TODO: error handling
//     // TODO: check input parameter type
//     pub fn set_commit_msg(&mut self, git_repository_head: Reference<'static>) {
//         let cbind = git_repository_head.peel_to_commit().unwrap();
//         let commit = cbind.message().unwrap().replace(['\n', '\"'], "");
//
//         self.commit_msg = match commit.char_indices().nth(120) {
//             None => commit.to_string(),
//             Some((idx, _)) => commit[..idx].to_string(),
//         }
//     }
//
//     // TODO: error handling
//     // TODO: check input parameter type
//     pub fn set_commit_datetime(&mut self, commit_object: Commit) {
//         let commit_date: i64 = commit_object.time().seconds();
//         self.commit_datetime = DateTime::from_utc(
//             NaiveDateTime::from_timestamp_opt(commit_date, 0).unwrap(),
//             Utc,
//         );
//     }
// }
