use crate::config::Config;
use crate::error::{Result, ScalpelError};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

pub struct ClaudeClient {
    client: Client,
    config: Config,
}

impl ClaudeClient {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()
            .map_err(|e| ScalpelError::ConfigError(e.to_string()))?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| ScalpelError::LlmApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    pub async fn analyze(
        &self,
        prompt: &str,
        system_prompt: Option<String>,
    ) -> Result<String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| ScalpelError::ConfigError("API key not configured".to_string()))?;

        let request = ClaudeRequest {
            model: self.config.model.clone(),
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            system: system_prompt,
        };

        let response = self.client
            .post(format!("{}/messages", self.config.endpoint))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ScalpelError::LlmApiError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ScalpelError::LlmApiError(
                format!("API returned {}: {}", status, error_text)
            ));
        }

        let claude_response: ClaudeResponse = response.json().await
            .map_err(|e| ScalpelError::LlmApiError(format!("Failed to parse response: {}", e)))?;

        // Extract text from content blocks
        let text = claude_response
            .content
            .iter()
            .filter_map(|block| block.text.clone())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }
}
