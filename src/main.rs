use clap::{Parser, arg};
use git2::{DiffOptions, Repository};

fn get_staged_diff(repo: &Repository) -> String {
    let index = repo.index().expect("Can't get index");
    let tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());
    let mut diff_opts = DiffOptions::new();
    let diff = repo
        .diff_tree_to_index(tree.as_ref(), Some(&index), Some(&mut diff_opts))
        .expect("Failed to generate diff");
    let mut buf = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_d, _h, _l| {
        buf.extend_from_slice(_l.content());
        true
    })
    .unwrap();
    String::from_utf8_lossy(&buf).to_string()
}

fn get_recent_commit_message(repo: &Repository) -> String {
    let mut revwalk = repo.revwalk().expect("Failed to get revwalk");
    revwalk.push_head().unwrap();
    let commits: Vec<String> = revwalk
        .take(3)
        .filter_map(|oid| oid.ok())
        .filter_map(|oid| repo.find_commit(oid).ok())
        .map(|commit| commit.message().unwrap_or("").trim().replace('"', "\\\""))
        .collect();
    commits.join("\n\n")
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("gpt-4o"))]
    model: String,
    #[arg(short, long, default_value_t = 3)]
    count: u8,
}

fn main() {
    let args = Args::parse();
    let repo = Repository::discover(".").expect("Not a git repository");
    let staged_diff = get_staged_diff(&repo);
    let commits = get_recent_commit_message(&repo);

    println!("current staged diff: {}", staged_diff);
    println!("current commits: {}", commits);
}
