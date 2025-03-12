use anyhow::{Context, Result};
use octocrab::models::Repository;
use log::debug;
use crate::github::GithubList;

pub struct AiSuggester {
    api_key: String,
}

impl AiSuggester {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn suggest_list(&self, repo: &Repository, available_lists: &[GithubList]) -> Result<Option<String>> {
        if available_lists.is_empty() {
            return Ok(None);
        }

        // Extract repository details
        let repo_name = repo.name.clone();
        let repo_description = repo.description.clone().unwrap_or_default();
        let topics: Vec<String> = repo.topics.clone().unwrap_or_default();
        
        // Format available list names and descriptions for the prompt
        let lists_desc = available_lists
            .iter()
            .map(|list| {
                let desc = list.description.clone().unwrap_or_default();
                format!("- {}: {}", list.name, desc)
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Create the prompt for the AI
        let prompt = format!(
            "I have a GitHub repository called '{}' with description: '{}'.\n\
            It has the following topics: {}.\n\n\
            I want to add it to one of my GitHub lists. Here are my available lists:\n{}\n\n\
            Which list would be the most appropriate for this repository? \
            Return just the list name without any explanation.",
            repo_name,
            repo_description,
            topics.join(", "),
            lists_desc
        );

        debug!("AI prompt: {}", prompt);

        // Create the client with API key
        let client = reqwest::Client::new();
        
        // Prepare the request body
        let request_body = serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": 50
        });

        // Send the request to OpenAI API
        let response = client.post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        // Parse the response
        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse OpenAI API response")?;

        // Extract the suggested list name from the response
        let suggested_list = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .map(|content| content.trim().to_string());

        // Verify the suggested list actually exists in the available lists
        let suggested_list = suggested_list.filter(|suggested| {
            available_lists.iter().any(|list| list.name == *suggested)
        });

        Ok(suggested_list)
    }
}