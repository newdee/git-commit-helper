use std::error::Error;

use async_openai::{
    Client as OpenAIClient,
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};
use clap::{Parser, arg};
use git2::{DiffOptions, Repository};

static PROMPT_TEMPLATE: &str = include_str!("prompt.txt");

fn get_staged_diff(repo: &Repository) -> String {
    let index = repo.index().expect("Can't get index");
    let tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());
    let mut diff_opts = DiffOptions::new();
    let diff = repo
        .diff_tree_to_index(tree.as_ref(), Some(&index), Some(&mut diff_opts))
        .expect("Failed to generate diff");
    let mut buf = Vec::new();
    if let Err(e) = diff.print(git2::DiffFormat::Patch, |_d, _h, _l| {
        buf.extend_from_slice(_l.content());
        true
    }) {
        eprintln!("failed to print diff: {}", e);
        return String::from("None");
    }
    String::from_utf8_lossy(&buf).to_string()
}

fn get_recent_commit_message(repo: &Repository) -> String {
    let mut revwalk = repo.revwalk().expect("Failed to get revwalk");
    if let Err(e) = revwalk.push_head() {
        eprintln!("Warning: Cannot find HEAD. Possibly no commits yet: {}", e);
        return String::from("None");
    };
    let commits: Vec<String> = revwalk
        .take(3)
        .filter_map(|oid| oid.ok())
        .filter_map(|oid| repo.find_commit(oid).ok())
        .map(|commit| commit.message().unwrap_or("").trim().replace('"', "\\\""))
        .collect();
    commits.join("\n\n")
}
fn commit_with_git(repo: &Repository, message: &str) -> Result<(), Box<dyn Error>> {
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

    let tree = repo.find_tree(tree_oid.id())?;

    let commit_oid = match parent_commit {
        Some(parent) => repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?,
        None => repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?,
    };

    println!("✅ Commit created: {}", commit_oid);
    Ok(())
}

async fn call_openai(prompt: &str, model: &str, max_token: u32) -> Result<String, Box<dyn Error>> {
    let base_url = std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "".to_string());
    let config = OpenAIConfig::default().with_api_base(base_url);
    let client = OpenAIClient::with_config(config);
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(max_token)
        .model(model)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into()])
        .build()?;
    let response = client.chat().create(request).await?;
    Ok(response
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default())
}

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
