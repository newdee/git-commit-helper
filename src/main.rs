// ************************************************************************** //
//                                                                            //
//                                                        :::      ::::::::   //
//   main.rs                                            :+:      :+:    :+:   //
//                                                    +:+ +:+         +:+     //
//   By: dfine <coding@dfine.tech>                  +#+  +:+       +#+        //
//                                                +#+#+#+#+#+   +#+           //
//   Created: 2025/05/06 19:11:51 by dfine             #+#    #+#             //
//   Updated: 2025/05/10 19:12:31 by dfine            ###   ########.fr       //
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

    println!("💬 Accept this commit message?");
    println!("    [Enter] to commit");
    println!("    [r]     to regenerate");
    println!("    [q]     to abort");
    print!("Your choice: ");
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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let repo = Repository::discover(".").expect("Not a git repository");
    let diff = get_staged_diff(&repo);
    let commits = get_recent_commit_message(&repo);
    let prompt = PROMPT_TEMPLATE
        .replace("{recent_commits}", &commits)
        .replace("{diff_context}", &diff);
    loop {
        match call_openai(&prompt, &args.model, args.max_token).await {
            Ok(commit_msg) => {
                println!("we generated the commit message is:\n{}\n", &commit_msg);
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
