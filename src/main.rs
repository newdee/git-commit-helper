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

use clap::{Parser, arg};

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

    #[arg(short, long, default_value_t = String::from("gpt-4o"))]
    model: String,

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

fn print_commit_msg(commit_msg: &str) {
    // blue
    println!("\x1b[34m================ COMMIT MESSAGE ================\x1b[0m");
    // green
    println!("\x1b[32m{commit_msg}\x1b[0m");
    // blue
    println!("\x1b[34m================================================\x1b[0m");
}

async fn summarize_diff_in_chunks(diff: &str, args: &Args) -> Result<String, Box<dyn Error>> {
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < diff.len() {
        let end = (start + args.chunk_size).min(diff.len());
        chunks.push(&diff[start..end]);
        start = end;
    }

    let mut part_summaries = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let prompt = format!(
            "You are a code change summarization assistant. Summarize the main goal and impact of the following code changes in a concise sentence:\n\nPart {} of the diff:\n{}\n\nSummary:",
            i + 1,
            chunk
        );

        match call_llm(&args.provider, &prompt, &args.model, args.max_token).await {
            Ok(summary) => {
                // println!("summary {i}: {summary}");
                part_summaries.push(summary.trim().to_string())
            }
            Err(e) => eprintln!("⚠️ summarizing chunk {} failed: {}", i + 1, e),
        }
    }
    let combined_summary = part_summaries.join("\n");
    // println!("{combined_summary}");
    Ok(combined_summary)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let repo = Repository::discover(".").expect("Not a git repository");
    let diff = get_staged_diff(&repo).unwrap_or_else(|| {
        eprintln!("⚠️  No staged changes found – nothing to commit.");
        std::process::exit(0);
    });
    let signkey = (!args.gpgsignkey.is_empty()).then_some(args.gpgsignkey.as_str());
    let commits = get_recent_commit_message(&repo).unwrap_or("None".to_string());
    let prompt = if diff.len() > args.chunk_size {
        println!(
            "Diff context is too large: {}, need to summarize",
            diff.len()
        );
        let summary = match summarize_diff_in_chunks(&diff, &args).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error summarizing diff: {e}");
                std::process::exit(0);
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
    loop {
        match call_llm(&args.provider, &prompt, &args.model, args.max_token).await {
            Ok(commit_msg) => {
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
            Err(e) => eprintln!("generate failed: {e}"),
        }
    }
}
