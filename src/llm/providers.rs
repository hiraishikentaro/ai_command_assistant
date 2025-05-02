use serde::{Deserialize, Serialize};
use std::fmt;

/// Available LLM providers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    /// OpenAI API (GPT models)
    OpenAI,
    /// Google Gemini API
    Gemini,
    /// Anthropic Claude API
    Claude,
}

impl Default for LlmProvider {
    fn default() -> Self {
        LlmProvider::OpenAI
    }
}

impl LlmProvider {
    /// Get all available providers as string slices
    pub fn all_providers() -> &'static [&'static str] {
        &["openai", "gemini", "claude"]
    }

    /// Get the provider's display name
    pub fn display_name(&self) -> &'static str {
        match self {
            LlmProvider::OpenAI => "OpenAI",
            LlmProvider::Gemini => "Google Gemini",
            LlmProvider::Claude => "Anthropic Claude",
        }
    }

    /// Get the provider's API key environment variable name
    pub fn api_key_env_var(&self) -> &'static str {
        match self {
            LlmProvider::OpenAI => "ACIA_OPENAI_API_KEY",
            LlmProvider::Gemini => "ACIA_GEMINI_API_KEY",
            LlmProvider::Claude => "ACIA_CLAUDE_API_KEY",
        }
    }

    /// Get the default models for the provider
    pub fn default_models(&self) -> Vec<&'static str> {
        match self {
            LlmProvider::OpenAI => vec!["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo"],
            LlmProvider::Gemini => vec!["gemini-pro", "gemini-1.5-pro"],
            LlmProvider::Claude => vec!["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"],
        }
    }

    /// Parse a provider from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(LlmProvider::OpenAI),
            "gemini" => Some(LlmProvider::Gemini),
            "claude" => Some(LlmProvider::Claude),
            _ => None,
        }
    }
}

impl fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Configuration for a specific LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key for this provider
    pub api_key: String,
    /// Whether this provider is the active one
    pub is_active: bool,
    /// The model to use for this provider
    pub model: String,
}

impl ProviderConfig {
    /// Create a new provider configuration
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            is_active: false,
            model,
        }
    }
}
