// ************************************************************************** //
//                                                                            //
//                                                        :::      ::::::::   //
//   llm.rs                                             :+:      :+:    :+:   //
//                                                    +:+ +:+         +:+     //
//   By: dfine <coding@dfine.tech>                  +#+  +:+       +#+        //
//                                                +#+#+#+#+#+   +#+           //
//   Created: 2025/05/10 19:12:36 by dfine             #+#    #+#             //
//   Updated: 2025/05/10 19:12:37 by dfine            ###   ########.fr       //
//                                                                            //
// ************************************************************************** //

use async_openai::{
    Client as OpenAIClient,
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};
use ollama_rs::{IntoUrlSealed, Ollama, generation::completion::request::GenerationRequest};
use std::error::Error;

/// Sends a prompt to the OpenAI chat API and returns the generated response as a string.
///
/// This function uses the `async-openai` crate to interact with a chat completion endpoint
/// (e.g., GPT-4, GPT-4o, GPT-3.5-turbo). The base URL can be overridden via the
/// `OPENAI_BASE_URL` environment variable.
///
/// # Arguments
///
/// * `prompt` - The text prompt to send to the model.
/// * `model` - The model ID to use (e.g., `"gpt-4o"`, `"gpt-3.5-turbo"`).
/// * `max_token` - Maximum number of tokens allowed in the response.
///
/// # Returns
///
/// A `Result` containing the generated string response on success, or an error on failure.
///
/// # Errors
///
/// This function will return an error if the request fails, the environment variable
/// is misconfigured, or if the response cannot be parsed correctly.
///
/// # Example
///
/// ```no_run
/// use git_commit_helper::call_openai;
///
/// #[tokio::main]
/// async fn main() {
///     let prompt = "Summarize the following diff...";
///     let model = "gpt-4o";
///     let max_token = 2048;
///
///     match call_openai(prompt, model, max_token).await {
///         Ok(response) => println!("LLM response: {}", response),
///         Err(e) => eprintln!("Error calling OpenAI: {}", e),
///     }
/// }
/// ```
pub async fn call_openai(
    prompt: &str,
    model: &str,
    max_token: u32,
) -> Result<String, Box<dyn Error>> {
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
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

pub async fn call_ollama(
    prompt: &str,
    model: &str,
    _max_token: u32,
) -> Result<String, Box<dyn Error>> {
    let base_url =
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let url = base_url.into_url()?;
    let client = Ollama::from_url(url);
    let request = client
        .generate(GenerationRequest::new(model.to_string(), prompt))
        .await?;
    Ok(request.response)
}

pub async fn call_llm(
    provider: &str,
    prompt: &str,
    model: &str,
    max_token: u32,
) -> Result<String, Box<dyn Error>> {
    match provider {
        "ollama" => call_ollama(prompt, model, max_token).await,
        _ => call_openai(prompt, model, max_token).await,
    }
}
