use crate::config::{Config, LlmProvider};
use crate::error::{Result, ScalpelError};
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Claude API structures
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

// Gemini API structures
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: String,
}

// Renamed from ClaudeClient to LlmClient
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
        match self.config.provider {
            LlmProvider::Claude => self.analyze_claude(prompt, system_prompt).await,
            LlmProvider::Gemini => self.analyze_gemini(prompt, system_prompt).await,
        }
    }

    async fn analyze_claude(
        &self,
        prompt: &str,
        system_prompt: Option<String>,
    ) -> Result<String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| ScalpelError::ConfigError("API key not configured".to_string()))?;

        let request = ClaudeRequest {
            model: self.config.model.clone(),
            max_tokens: 4096,
            messages: vec![ClaudeMessage {
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

    async fn analyze_gemini(
        &self,
        prompt: &str,
        system_prompt: Option<String>,
    ) -> Result<String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| ScalpelError::ConfigError("API key not configured".to_string()))?;

        let system_instruction = system_prompt.map(|text| GeminiSystemInstruction {
            parts: vec![GeminiPart { text }],
        });

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
            system_instruction,
            generation_config: GeminiGenerationConfig {
                temperature: 0.7,
                max_output_tokens: 8192,
            },
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.config.endpoint,
            self.config.model,
            api_key
        );

        let response = self.client
            .post(&url)
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

        let gemini_response: GeminiResponse = response.json().await
            .map_err(|e| ScalpelError::LlmApiError(format!("Failed to parse response: {}", e)))?;

        // Extract text from candidates
        let text = gemini_response
            .candidates
            .into_iter()
            .flat_map(|c| c.content.parts)
            .map(|p| p.text)
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }
}
