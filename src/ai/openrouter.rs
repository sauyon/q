use crate::ai::AIProvider;
use crate::config::OpenRouterConfig;
use crate::models::{CommandSuggestion, SystemContext};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct OpenRouterProvider {
    config: OpenRouterConfig,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

impl OpenRouterProvider {
    pub fn new(config: OpenRouterConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    fn build_system_prompt(context: &SystemContext) -> String {
        format!(
            r#"You are a command-line assistant that helps users by generating shell commands.

System Information:
- OS: {}
- Shell: {}
- Current Directory: {}

Your task is to:
1. Understand the user's intent from their natural language query
2. Generate the appropriate shell command for their system
3. Provide a clear explanation of what the command does
4. Warn about potentially destructive operations

Respond ONLY with a JSON object in this exact format:
{{
  "command": "the actual command to run",
  "explanation": "clear explanation of what this command does",
  "warning": "optional warning about destructive operations, or null if safe"
}}

Important:
- Generate commands appropriate for the {} shell on {}
- Be concise but clear in explanations
- Always include warnings for commands that delete, modify, or move files
- If the request is ambiguous, make reasonable assumptions but mention them in the explanation
- If you need the user to provide specific values (like IDs, names, paths), use the syntax {{{{VARIABLE_NAME}}}} (e.g., {{{{VPC_ID}}}}, {{{{FILE_PATH}}}}). Do NOT use generic placeholders like <vpc-id> or [name].
- Return ONLY the JSON object, no other text"#,
            context.os, context.shell, context.current_dir, context.shell, context.os
        )
    }
}

#[async_trait]
impl AIProvider for OpenRouterProvider {
    async fn generate_command(
        &self,
        query: &str,
        context: &SystemContext,
    ) -> Result<CommandSuggestion> {
        let system_prompt = Self::build_system_prompt(context);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: query.to_string(),
                },
            ],
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenRouter API error ({}): {}", status, error_text);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse OpenRouter response")?;

        let content = chat_response
            .choices
            .first()
            .context("No response from AI")?
            .message
            .content
            .clone();

        // Parse the JSON response from the AI
        let suggestion: CommandSuggestion = serde_json::from_str(&content)
            .context("Failed to parse AI response as JSON. Response was not in expected format.")?;

        Ok(suggestion)
    }
}
