use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    Claude,
    Gemini,
}

fn default_provider() -> LlmProvider {
    LlmProvider::Claude
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLM provider to use (claude or gemini)
    #[serde(default = "default_provider")]
    pub provider: LlmProvider,

    /// API key for the selected provider
    pub api_key: Option<String>,

    /// Model to use
    #[serde(default = "default_model")]
    pub model: String,

    /// API endpoint (provider-specific)
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// Path to vmlinux debug symbols
    pub vmlinux_path: Option<PathBuf>,

    /// Enable safety guards for mutations
    #[serde(default = "default_true")]
    pub safety_guards: bool,

    /// Require confirmation for dangerous operations
    #[serde(default = "default_true")]
    pub require_confirmation: bool,

    /// Maximum number of active probes
    #[serde(default = "default_max_probes")]
    pub max_probes: usize,

    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_model() -> String {
    "claude-3-5-sonnet-20241022".to_string()
}

fn default_endpoint() -> String {
    "https://api.anthropic.com/v1".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_probes() -> usize {
    16
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for Config {
    fn default() -> Self {
        // Try to detect provider from environment
        let (provider, api_key) = if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            (LlmProvider::Gemini, Some(key))
        } else if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            (LlmProvider::Claude, Some(key))
        } else {
            (LlmProvider::Claude, None)
        };

        Self {
            provider: provider.clone(),
            api_key,
            model: match provider {
                LlmProvider::Claude => "claude-3-5-sonnet-20241022".to_string(),
                LlmProvider::Gemini => "gemini-2.0-flash-exp".to_string(),
            },
            endpoint: match provider {
                LlmProvider::Claude => "https://api.anthropic.com/v1".to_string(),
                LlmProvider::Gemini => "https://generativelanguage.googleapis.com/v1beta".to_string(),
            },
            vmlinux_path: None,
            safety_guards: true,
            require_confirmation: true,
            max_probes: 16,
            log_level: default_log_level(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let mut config: Config = toml::from_str(&contents)
                .context("Failed to parse config file")?;

            // Override with environment variable if present based on provider
            match config.provider {
                LlmProvider::Claude => {
                    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
                        config.api_key = Some(api_key);
                    }
                }
                LlmProvider::Gemini => {
                    if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
                        config.api_key = Some(api_key);
                    }
                }
            }

            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, contents)
            .context("Failed to write config file")?;

        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;
        Ok(config_dir.join("scalpel").join("config.toml"))
    }

    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_none() {
            let env_var = match self.provider {
                LlmProvider::Claude => "ANTHROPIC_API_KEY",
                LlmProvider::Gemini => "GEMINI_API_KEY",
            };
            anyhow::bail!(
                "API key not configured. Set {} environment variable or configure in ~/.config/scalpel/config.toml",
                env_var
            );
        }
        Ok(())
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Scalpel Configuration:")?;
        writeln!(f, "  Provider: {:?}", self.provider)?;
        writeln!(f, "  Model: {}", self.model)?;
        writeln!(f, "  Endpoint: {}", self.endpoint)?;
        writeln!(f, "  API Key: {}", if self.api_key.is_some() { "configured" } else { "not set" })?;
        writeln!(f, "  VMLinux Path: {}", self.vmlinux_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "auto-detect".to_string()))?;
        writeln!(f, "  Safety Guards: {}", self.safety_guards)?;
        writeln!(f, "  Require Confirmation: {}", self.require_confirmation)?;
        writeln!(f, "  Max Probes: {}", self.max_probes)?;
        writeln!(f, "  Log Level: {}", self.log_level)?;
        Ok(())
    }
}
