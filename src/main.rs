// ************************************************************************** //
//                                                                            //
//                                                        :::      ::::::::   //
//   main.rs                                            :+:      :+:    :+:   //
//                                                    +:+ +:+         +:+     //
//   By: dfine <coding@dfine.tech>                  +#+  +:+       +#+        //
//                                                +#+#+#+#+#+   +#+           //
//   Created: 2025/05/06 19:11:51 by dfine             #+#    #+#             //
//   Updated: 2025/05/11 00:38:41 by dfine            ###   ########.fr       //
//                                                                            //
// ************************************************************************** //

use git_commit_helper::{
    commit_with_git, get_recent_commit_message, get_staged_diff, llm::call_llm,
};
use git2::Repository;
use std::error::Error;

use clap::Parser;

static PROMPT_TEMPLATE: &str = include_str!("prompt.txt");
static CHUNK_PROMPT_TEMPLATE: &str = include_str!("prompt_chunked.txt");

enum UserChoice {
    Commit,
    Regenerate,
    Abort,
}
fn prompt_user_action() -> Result<UserChoice, Box<dyn Error>> {
    use std::io::{Write, stdin, stdout};
    println!("\x1b[1;36m💬 Accept this commit message?\x1b[0m"); // bold cyan
    println!("  \x1b[1;32m[Enter]\x1b[0m to commit");
    println!("  \x1b[1;33m[r]    \x1b[0m to regenerate");
    println!("  \x1b[1;31m[q]    \x1b[0m to abort");
    print!("\x1b[1m👉 Your choice: \x1b[0m ");
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    match trimmed.as_str() {
        "" => Ok(UserChoice::Commit),
        "r" => Ok(UserChoice::Regenerate),
        "q" => Ok(UserChoice::Abort),
        _ => {
            println!("⚠️ Invalid input.");
            prompt_user_action()
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("openai"))]
    provider: String,

    /// Model ID. Defaults per provider when omitted
    /// (openai: gpt-4o, anthropic: claude-opus-4-8, ollama: llama3.2).
    #[arg(short, long)]
    model: Option<String>,

    #[arg(long, default_value_t = false)]
    gpgsign: bool,

    #[arg(long, default_value_t = String::new())]
    gpgsignkey: String,
    //#[arg(short, long, default_value_t = 3)]
    //count: u8,
    #[arg(long, default_value_t = 2048_u32)]
    max_token: u32,

    #[arg(long, default_value_t = 200000_usize)]
    chunk_size: usize,
}

fn default_model(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "claude-opus-4-8",
        "ollama" => "llama3.2",
        _ => "gpt-4o",
    }
}

fn print_commit_msg(commit_msg: &str) {
    // blue
    println!("\x1b[34m================ COMMIT MESSAGE ================\x1b[0m");
    // green
    println!("\x1b[32m{commit_msg}\x1b[0m");
    // blue
    println!("\x1b[34m================================================\x1b[0m");
}

/// Returns the byte offsets at which each file section (`diff --git ...`) begins,
/// plus a trailing sentinel equal to `diff.len()`. File `k` spans `bounds[k]..bounds[k+1]`.
/// Offsets are always valid UTF-8 char boundaries (the marker is ASCII).
fn file_bounds(diff: &str) -> Vec<usize> {
    let marker = "diff --git ";
    let mut bounds = Vec::new();
    if diff.starts_with(marker) {
        bounds.push(0);
    }
    bounds.extend(
        diff.match_indices(&format!("\n{marker}"))
            .map(|(i, _)| i + 1),
    );
    if bounds.is_empty() {
        // No recognizable file markers: treat the whole diff as one section.
        bounds.push(0);
    }
    bounds.push(diff.len());
    bounds
}

/// Splits a diff into chunks no larger than `chunk_size` bytes. Consecutive whole files
/// are packed into the same chunk to keep the chunk count (and thus the number of LLM
/// calls) minimal. A single file larger than `chunk_size` is split on UTF-8 char
/// boundaries (never mid-character, which would panic).
fn split_diff(diff: &str, chunk_size: usize) -> Vec<&str> {
    let bounds = file_bounds(diff);
    let file_count = bounds.len() - 1;
    let mut chunks = Vec::new();
    let mut i = 0;
    while i < file_count {
        let start = bounds[i];
        // Greedily pack whole files while they fit; always take at least one file.
        let mut j = i + 1;
        while j < file_count && bounds[j + 1] - start <= chunk_size {
            j += 1;
        }
        let end = bounds[j];

        if j == i + 1 && end - start > chunk_size {
            // A single file exceeds chunk_size: sub-split it on char boundaries.
            let mut s = start;
            while s < end {
                let mut e = (s + chunk_size).min(end);
                while e < end && !diff.is_char_boundary(e) {
                    e -= 1;
                }
                if e <= s {
                    // No progress (multibyte char at `s`, tiny chunk_size): step forward.
                    e = s + 1;
                    while e < end && !diff.is_char_boundary(e) {
                        e += 1;
                    }
                }
                chunks.push(&diff[s..e]);
                s = e;
            }
        } else {
            chunks.push(&diff[start..end]);
        }
        i = j;
    }
    chunks
}

async fn summarize_diff_in_chunks(
    diff: &str,
    provider: &str,
    model: &str,
    max_token: u32,
    chunk_size: usize,
) -> Result<String, Box<dyn Error>> {
    use futures::stream::StreamExt;
    // Cap in-flight requests so a diff that splits into many chunks doesn't fan out
    // into an unbounded burst of concurrent API calls (rate limits, cost).
    const MAX_CONCURRENT: usize = 8;

    let chunks = split_diff(diff, chunk_size);

    // Summarize chunks concurrently (bounded), preserving order.
    let tasks = chunks.into_iter().enumerate().map(|(i, chunk)| async move {
        let prompt = format!(
            "You are a code change summarization assistant. Summarize the main goal and impact of the following code changes in a concise sentence:\n\nPart {} of the diff:\n{}\n\nSummary:",
            i + 1,
            chunk
        );
        (i, call_llm(provider, &prompt, model, max_token).await)
    });
    let results: Vec<_> = futures::stream::iter(tasks)
        .buffered(MAX_CONCURRENT)
        .collect()
        .await;

    let mut part_summaries = Vec::new();
    for (i, res) in results {
        match res {
            Ok(summary) => part_summaries.push(summary.trim().to_string()),
            Err(e) => eprintln!("⚠️ summarizing chunk {} failed: {}", i + 1, e),
        }
    }

    if part_summaries.is_empty() {
        return Err("all diff chunks failed to summarize".into());
    }
    Ok(part_summaries.join("\n"))
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let repo = Repository::discover(".").expect("Not a git repository");
    let diff = match get_staged_diff(&repo) {
        Ok(Some(diff)) => diff,
        Ok(None) => {
            eprintln!("⚠️  No staged changes found – nothing to commit.");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Failed to read staged diff: {e}");
            std::process::exit(1);
        }
    };
    let signkey = (!args.gpgsignkey.is_empty()).then_some(args.gpgsignkey.as_str());
    let model = args
        .model
        .clone()
        .unwrap_or_else(|| default_model(&args.provider).to_string());
    let commits = get_recent_commit_message(&repo).unwrap_or("None".to_string());
    let prompt = if diff.len() > args.chunk_size {
        println!(
            "Diff context is too large: {}, need to summarize",
            diff.len()
        );
        let summary = match summarize_diff_in_chunks(
            &diff,
            &args.provider,
            &model,
            args.max_token,
            args.chunk_size,
        )
        .await
        {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error summarizing diff: {e}");
                std::process::exit(1);
            }
        };
        CHUNK_PROMPT_TEMPLATE
            .replace("{recent_commits}", &commits)
            .replace("{diff_context}", &summary)
    } else {
        PROMPT_TEMPLATE
            .replace("{recent_commits}", &commits)
            .replace("{diff_context}", &diff)
    };
    // Bound automatic (non-user-driven) regenerations so a model that keeps
    // returning empty output can't spin forever without any input.
    const MAX_EMPTY_RETRIES: u32 = 3;
    let mut empty_retries = 0;
    loop {
        match call_llm(&args.provider, &prompt, &model, args.max_token).await {
            Ok(commit_msg) => {
                if commit_msg.trim().is_empty() {
                    empty_retries += 1;
                    if empty_retries > MAX_EMPTY_RETRIES {
                        eprintln!("❌ Model returned an empty commit message repeatedly. Aborting.");
                        std::process::exit(1);
                    }
                    eprintln!("⚠️ Model returned an empty commit message. Regenerating...\n");
                    continue;
                }
                empty_retries = 0;
                print_commit_msg(&commit_msg);
                match prompt_user_action() {
                    Ok(UserChoice::Abort) => {
                        println!("❎ Commit aborted.");
                        break;
                    }
                    Ok(UserChoice::Commit) => {
                        if let Err(e) = commit_with_git(&repo, &commit_msg, args.gpgsign, signkey) {
                            eprintln!("❌ Commit failed: {e}");
                        }
                        break;
                    }
                    Ok(UserChoice::Regenerate) => {
                        println!("🔁 Regenerating...\n");
                        continue;
                    }
                    Err(e) => {
                        eprintln!("❌ Input error: {e}");
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Generate failed: {e}");
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{default_model, split_diff};

    #[test]
    fn split_diff_handles_multibyte_without_panicking() {
        // Multibyte (each '世' is 3 bytes); a byte-based split would land mid-char.
        let diff = "diff --git a/f b/f\n世界世界世界世界世界\n";
        let chunks = split_diff(diff, 5);
        assert!(chunks.len() > 1, "expected the file to be split");
        // Every chunk is valid UTF-8 (guaranteed by &str) and they reassemble losslessly.
        assert_eq!(chunks.concat(), diff);
    }

    #[test]
    fn split_diff_packs_files_and_respects_boundaries() {
        let diff = "diff --git a/x b/x\n+one\ndiff --git a/y b/y\n+two\n";
        // Large chunk: both files pack into a single chunk.
        let one = split_diff(diff, 1000);
        assert_eq!(one.len(), 1);
        assert_eq!(one.concat(), diff);
        // Chunk sized to a single file: split on the file boundary, not mid-file.
        let first_file_len = "diff --git a/x b/x\n+one\n".len();
        let two = split_diff(diff, first_file_len);
        assert_eq!(two.len(), 2);
        assert!(two[0].starts_with("diff --git a/x"));
        assert!(two[1].starts_with("diff --git a/y"));
        assert_eq!(two.concat(), diff);
    }

    #[test]
    fn default_model_is_provider_specific() {
        assert_eq!(default_model("anthropic"), "claude-opus-4-8");
        assert_eq!(default_model("ollama"), "llama3.2");
        assert_eq!(default_model("openai"), "gpt-4o");
        assert_eq!(default_model("anything-else"), "gpt-4o");
    }
}
