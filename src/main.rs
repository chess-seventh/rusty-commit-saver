use git2::Repository;

fn main() {
    let git_repo = Repository::discover("./").unwrap();
    let head = git_repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap().id();
    let branch_name = head.shorthand().unwrap();
    println!("{commit:?}");
    println!("{branch_name:?}");
}
