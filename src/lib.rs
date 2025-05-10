// ************************************************************************** //
//                                                                            //
//                                                        :::      ::::::::   //
//   lib.rs                                             :+:      :+:    :+:   //
//                                                    +:+ +:+         +:+     //
//   By: dfine <coding@dfine.tech>                  +#+  +:+       +#+        //
//                                                +#+#+#+#+#+   +#+           //
//   Created: 2025/05/10 19:12:41 by dfine             #+#    #+#             //
//   Updated: 2025/05/10 19:12:43 by dfine            ###   ########.fr       //
//                                                                            //
// ************************************************************************** //

//! # git-commit-helper
//!
/// `git-commit-helper` is a library designed to simplify the process of generating high-quality
/// Git commit messages using large language models like OpenAI's GPT.
///
/// It provides tools to:
/// - Extract staged diffs and recent commit messages from a Git repository
/// - Generate commit messages via OpenAI chat completion APIs
/// - Automatically create commits with the generated messages
///
/// ## Example use cases
/// - Enhance developer workflows with AI-generated commit messages
/// - Integrate LLM commit generation into custom Git UIs or bots
pub mod git;
pub mod llm;

/// Re-exports for convenient use in consumers of the library
pub use git::{commit_with_git, get_recent_commit_message, get_staged_diff};
pub use llm::call_openai;
