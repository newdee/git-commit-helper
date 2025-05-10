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

use git_commit_helper::{call_openai, commit_with_git, get_recent_commit_message, get_staged_diff};
use git2::Repository;
use std::error::Error;

use clap::{Parser, arg};

static PROMPT_TEMPLATE: &str = include_str!("prompt.txt");

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
    #[arg(short, long, default_value_t = String::from("gpt-4o"))]
    model: String,
    //#[arg(short, long, default_value_t = 3)]
    //count: u8,
    #[arg(long, default_value_t = 2048_u32)]
    max_token: u32,
}

fn print_commit_msg(commit_msg: &str) {
    // blue
    println!("\x1b[34m================ COMMIT MESSAGE ================\x1b[0m");
    // green
    println!("\x1b[32m{}\x1b[0m", commit_msg);
    // blue
    println!("\x1b[34m================================================\x1b[0m");
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let repo = Repository::discover(".").expect("Not a git repository");
    let diff = get_staged_diff(&repo).unwrap_or_else(|| {
        eprintln!("⚠️  No staged changes found – nothing to commit.");
        std::process::exit(0);
    });
    let commits = get_recent_commit_message(&repo).unwrap_or("None".to_string());
    let prompt = PROMPT_TEMPLATE
        .replace("{recent_commits}", &commits)
        .replace("{diff_context}", &diff);
    loop {
        match call_openai(&prompt, &args.model, args.max_token).await {
            Ok(commit_msg) => {
                print_commit_msg(&commit_msg);
                match prompt_user_action() {
                    Ok(UserChoice::Abort) => {
                        println!("❎ Commit aborted.");
                        break;
                    }
                    Ok(UserChoice::Commit) => {
                        if let Err(e) = commit_with_git(&repo, &commit_msg) {
                            eprintln!("❌ Commit failed: {}", e);
                        }
                        break;
                    }
                    Ok(UserChoice::Regenerate) => {
                        println!("🔁 Regenerating...\n");
                        continue;
                    }
                    Err(e) => {
                        eprintln!("❌ Input error: {}", e);
                        break;
                    }
                }
            }
            Err(e) => eprintln!("generate failed: {}", e),
        }
    }
}
