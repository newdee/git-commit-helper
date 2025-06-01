// ************************************************************************** //
//                                                                            //
//                                                        :::      ::::::::   //
//   git.rs                                             :+:      :+:    :+:   //
//                                                    +:+ +:+         +:+     //
//   By: dfine <coding@dfine.tech>                  +#+  +:+       +#+        //
//                                                +#+#+#+#+#+   +#+           //
//   Created: 2025/05/10 19:12:46 by dfine             #+#    #+#             //
//   Updated: 2025/06/02 02:00:48 by dfine            ###   ########.fr       //
//                                                                            //
// ************************************************************************** //

use git2::{DiffOptions, Repository};
use std::{
    error::Error,
    io::Write,
    process::{Command, Stdio},
};

/// Returns the staged diff of the current Git repository (i.e., changes staged for commit).
///
/// This compares the staged index against the current `HEAD`.
///
/// # Arguments
///
/// * `repo` - A reference to an open `git2::Repository` instance.
///
/// # Returns
///
/// A `String` containing the unified diff. If the diff cannot be generated, it returns `"None"`.
///
/// # Example
///
/// ```
/// use git_commit_helper::get_staged_diff;
/// use git2::Repository;
///
/// let repo = Repository::discover(".").expect("Not a git repository");
/// let diff = get_staged_diff(&repo);
/// println!("{:?}", diff);
/// ```
pub fn get_staged_diff(repo: &Repository) -> Option<String> {
    let index = repo.index().ok()?;
    let tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());
    let mut diff_opts = DiffOptions::new();
    let diff = repo
        .diff_tree_to_index(tree.as_ref(), Some(&index), Some(&mut diff_opts))
        .ok()?;
    let mut buf = Vec::new();
    if let Err(e) = diff.print(git2::DiffFormat::Patch, |_d, _h, _l| {
        buf.extend_from_slice(_l.content());
        true
    }) {
        eprintln!("failed to print diff: {}", e);
        return None;
    }
    let result = String::from_utf8_lossy(&buf).to_string();
    if result.trim().is_empty() {
        return None;
    }
    Some(result)
}

/// Returns the messages of the most recent commits (up to 3).
///
/// Useful for providing context to an LLM or for generating summaries.
///
/// # Arguments
///
/// * `repo` - A reference to an open `git2::Repository` instance.
///
/// # Returns
///
/// A newline-separated string of the latest commit messages. If no commits exist, returns `"None"`.
///
/// # Example
///
/// ```
/// use git_commit_helper::get_recent_commit_message;
/// use git2::Repository;
///
/// let repo = Repository::discover(".").expect("Not a git repository");
/// let messages = get_recent_commit_message(&repo);
/// println!("{:?}", messages);
/// ```
pub fn get_recent_commit_message(repo: &Repository) -> Option<String> {
    let mut revwalk = repo.revwalk().ok()?;
    revwalk.push_head().ok()?;
    let commits: Vec<String> = revwalk
        .take(3)
        .filter_map(|oid| oid.ok())
        .filter_map(|oid| repo.find_commit(oid).ok())
        .map(|commit| commit.message().unwrap_or("").trim().replace('"', "\\\""))
        .collect();
    if commits.is_empty() {
        return None;
    }
    Some(commits.join("\n\n"))
}

pub fn gpg_sign(data: &[u8], key: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("gpg");
    cmd.args(["--armor", "--detach-sign"]);

    if let Some(k) = key {
        cmd.args(["--local-user", k]);
    }

    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
    child.stdin.as_mut().unwrap().write_all(data)?;
    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(format!(
            "GPG signing failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

/// Commits the currently staged changes with the provided commit message.
///
/// This function handles both initial and regular commits, constructing the commit tree
/// and linking to the correct parent if available.
///
/// # Arguments
///
/// * `repo` - A reference to an open `git2::Repository` instance.
/// * `message` - The commit message to use.
///
/// # Errors
///
/// Returns a boxed `Error` if Git operations (e.g., getting the index, writing tree, or committing) fail.
///
/// # Example
///
/// ```
/// use git_commit_helper::commit_with_git;
/// use git2::Repository;
///
/// let repo = Repository::discover(".").expect("Not a git repository");
/// let message = "Add README and initial setup";
/// if let Err(err) = commit_with_git(&repo, message) {
///     eprintln!("Commit failed: {}", err);
/// }
/// ```
pub fn commit_with_git(
    repo: &Repository,
    message: &str,
    gpgsign: bool,
    signkey: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let sig = repo.signature()?;

    let tree_oid = {
        let mut index = repo.index()?;
        let oid = index.write_tree()?;
        repo.find_tree(oid)?
    };

    let head = repo.head().ok();
    let parent_commit = head
        .as_ref()
        .and_then(|h| h.target())
        .and_then(|oid| repo.find_commit(oid).ok());
    let parents = parent_commit.iter().collect::<Vec<_>>();

    let tree = repo.find_tree(tree_oid.id())?;
    let buf = repo.commit_create_buffer(&sig, &sig, message, &tree, &parents)?;

    if !gpgsign {
        let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
        println!("✅ Commit created: {}", commit_oid);
        return Ok(());
    }
    let signature = gpg_sign(&buf, signkey);
    let commit_oid =
        repo.commit_signed(buf.as_str().unwrap(), signature.unwrap().as_str(), None)?;
    // let commit = repo.find_commit(commit_oid)?;
    // repo.branch(head.unwrap().shorthand().unwrap(), &commit, false)?;
    repo.reference(
        head.unwrap().name().unwrap(),
        commit_oid,
        true,
        "update main",
    )?;

    println!("✅ Commit created: {}", commit_oid);
    Ok(())
}
