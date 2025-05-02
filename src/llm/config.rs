use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::llm::error::LlmError;
use crate::llm::providers::{LlmProvider, ProviderConfig};
use dirs;
use log::debug;
use serde::{Deserialize, Serialize};
use toml;

/// Configuration for LLM API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub timeout_seconds: u64,

    #[serde(default)]
    pub provider: LlmProvider,

    #[serde(default, skip_serializing)]
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: 500,
            temperature: 0.2,
            top_p: 1.0,
            timeout_seconds: 30,
            provider: LlmProvider::OpenAI,
            providers: HashMap::new(),
        }
    }
}

impl LlmConfig {
    /// Create a new config instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Load the config from file or environment variables
    pub fn load() -> Result<Self, LlmError> {
        let mut config = Self::default();

        // Try to load from config file first
        if let Some(config_path) = config.get_config_file_path() {
            if config_path.exists() {
                match fs::read_to_string(&config_path) {
                    Ok(content) => {
                        match toml::from_str::<LlmConfig>(&content) {
                            Ok(file_config) => {
                                config = file_config;
                                debug!("Loaded config from file: {:?}", config_path);

                                // Load providers config
                                if let Some(providers_section) = content.find("[providers]") {
                                    let providers_content = &content[providers_section..];
                                    config.parse_providers_config(providers_content);
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse config file: {}", e);
                                // Still try to extract API key if parsing failed
                                if let Some(api_key) = config.extract_api_key_from_config(&content)
                                {
                                    config.api_key = api_key;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Failed to read config file: {}", e);
                    }
                }
            }
        }

        // Try to load API keys from environment variables
        for provider in &[
            LlmProvider::OpenAI,
            LlmProvider::Gemini,
            LlmProvider::Claude,
        ] {
            if let Ok(api_key) = env::var(provider.api_key_env_var()) {
                if !api_key.is_empty() {
                    let model = provider.default_models()[0].to_string();
                    let provider_config = ProviderConfig::new(api_key.clone(), model);

                    let provider_key = format!("{:?}", provider).to_lowercase();
                    config.providers.insert(provider_key, provider_config);

                    // Set the current provider's API key if it matches the active provider
                    if *provider == config.provider {
                        config.api_key = api_key;
                        debug!("Loaded API key for {} from environment variable", provider);
                    }
                }
            }
        }

        // Set active provider config
        let provider_str = format!("{:?}", config.provider).to_lowercase();
        if let Some(provider_config) = config.providers.get_mut(&provider_str) {
            provider_config.is_active = true;

            // Set primary API key and model from the active provider
            if config.api_key.is_empty() {
                config.api_key = provider_config.api_key.clone();
            }
            config.model = provider_config.model.clone();
        }

        // Ensure we have an API key for the active provider
        if config.api_key.is_empty() {
            return Err(LlmError::ApiKeyNotSet);
        }

        Ok(config)
    }

    /// Load API key - for backward compatibility
    pub fn load_api_key(&mut self) -> Result<(), LlmError> {
        // Load from environment variable
        let env_var = self.provider.api_key_env_var();
        if let Ok(api_key) = env::var(env_var) {
            if !api_key.is_empty() {
                self.api_key = api_key;
                debug!("Loaded API key from environment variable");
                return Ok(());
            }
        }

        // Load from config file
        if let Some(config_path) = self.get_config_file_path() {
            if config_path.exists() {
                match fs::read_to_string(&config_path) {
                    Ok(content) => {
                        if let Some(api_key) = self.extract_api_key_from_config(&content) {
                            self.api_key = api_key;
                            debug!("Loaded API key from config file: {:?}", config_path);
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        debug!("Failed to read config file: {}", e);
                    }
                }
            }
        }

        // Error if API key not found
        if self.api_key.is_empty() {
            return Err(LlmError::ApiKeyNotSet);
        }

        Ok(())
    }

    /// Parse providers section from config
    fn parse_providers_config(&mut self, content: &str) {
        let mut current_provider: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for provider section
            if line.starts_with('[') && line.ends_with(']') {
                let section = line.trim_start_matches('[').trim_end_matches(']');
                if section.starts_with("providers.") {
                    let provider_name = section.trim_start_matches("providers.").to_string();
                    current_provider = Some(provider_name);

                    // Create provider config if it doesn't exist
                    if !self
                        .providers
                        .contains_key(&current_provider.clone().unwrap())
                    {
                        let model = match LlmProvider::from_str(&current_provider.clone().unwrap())
                        {
                            Some(provider) => provider.default_models()[0].to_string(),
                            None => "unknown".to_string(),
                        };

                        self.providers.insert(
                            current_provider.clone().unwrap(),
                            ProviderConfig::new(String::new(), model),
                        );
                    }
                }
                continue;
            }

            // Process key-value pairs
            if let Some(provider_name) = &current_provider {
                if let Some(provider_config) = self.providers.get_mut(provider_name) {
                    if let Some((key, value)) = parse_key_value(line) {
                        match key.as_str() {
                            "api_key" => {
                                provider_config.api_key = value;
                            }
                            "model" => {
                                provider_config.model = value;
                            }
                            "is_active" => {
                                provider_config.is_active = value.to_lowercase() == "true";

                                // Set this as the active provider
                                if provider_config.is_active {
                                    if let Some(provider) = LlmProvider::from_str(provider_name) {
                                        self.provider = provider;
                                        // Update main config with this provider's settings
                                        self.api_key = provider_config.api_key.clone();
                                        self.model = provider_config.model.clone();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    /// Get config file path
    fn get_config_file_path(&self) -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| {
            let mut path = config_dir;
            path.push("acia");
            path.push("config.toml");
            path
        })
    }

    /// Extract API key from config content (for backward compatibility)
    fn extract_api_key_from_config(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("openai_api_key") || line.starts_with("api_key") {
                if let Some((_, value)) = parse_key_value(line) {
                    if !value.is_empty() {
                        return Some(value);
                    }
                }
            }
        }
        None
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), LlmError> {
        if let Some(config_path) = self.get_config_file_path() {
            // Create directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    LlmError::ConfigError(format!("Failed to create config directory: {}", e))
                })?;
            }

            // Serialize main config
            let mut config_content = toml::to_string(&self)
                .map_err(|e| LlmError::ConfigError(format!("Failed to serialize config: {}", e)))?;

            // Add providers section
            if !self.providers.is_empty() {
                config_content.push_str("\n[providers]\n");

                for (provider_name, provider_config) in &self.providers {
                    config_content.push_str(&format!("\n[providers.{}]\n", provider_name));
                    config_content
                        .push_str(&format!("api_key = \"{}\"\n", provider_config.api_key));
                    config_content.push_str(&format!("model = \"{}\"\n", provider_config.model));
                    config_content
                        .push_str(&format!("is_active = {}\n", provider_config.is_active));
                }
            }

            // Write to file
            let mut file = fs::File::create(config_path).map_err(|e| {
                LlmError::ConfigError(format!("Failed to create config file: {}", e))
            })?;

            file.write_all(config_content.as_bytes()).map_err(|e| {
                LlmError::ConfigError(format!("Failed to write config file: {}", e))
            })?;

            debug!("Saved config to file");
            return Ok(());
        }

        Err(LlmError::ConfigError(
            "Failed to get config file path".to_string(),
        ))
    }

    /// Set provider and its config
    pub fn set_provider(
        &mut self,
        provider: LlmProvider,
        api_key: String,
        model: String,
    ) -> &mut Self {
        // Update main config
        self.provider = provider.clone();
        self.api_key = api_key.clone();
        self.model = model.clone();

        // Add or update provider config
        let provider_str = format!("{:?}", provider).to_lowercase();
        let mut provider_config = ProviderConfig::new(api_key, model);
        provider_config.is_active = true;

        // Deactivate other providers
        for (_, config) in self.providers.iter_mut() {
            config.is_active = false;
        }

        // Add/update the provider
        self.providers.insert(provider_str, provider_config);

        self
    }

    /// Set model for current provider
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        let model_str = model.into();
        self.model = model_str.clone();

        // Update provider config if it exists
        let provider_str = format!("{:?}", self.provider).to_lowercase();
        if let Some(config) = self.providers.get_mut(&provider_str) {
            config.model = model_str;
        }

        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}

// Helper function to parse key-value pairs from config
fn parse_key_value(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.split('=').collect();
    if parts.len() >= 2 {
        let key = parts[0].trim().to_string();
        let value = parts[1..].join("=").trim().to_string();
        // Remove quotes
        let value = value.trim_matches('"').trim_matches('\'').to_string();
        return Some((key, value));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.max_tokens, 500);
        assert_eq!(config.temperature, 0.2);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.provider, LlmProvider::OpenAI);
    }

    #[test]
    fn test_builder_pattern() {
        let config = LlmConfig::new()
            .with_model("gpt-4")
            .with_temperature(0.7)
            .with_max_tokens(1000)
            .with_timeout(60);

        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_tokens, 1000);
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_extract_api_key_from_config() {
        let config = LlmConfig::default();

        // Normal case
        let content = r#"
        [openai]
        openai_api_key = "sk-1234567890abcdef"
        "#;
        assert_eq!(
            config.extract_api_key_from_config(content),
            Some("sk-1234567890abcdef".to_string())
        );

        // Single quotes case
        let content = r#"
        api_key = 'sk-abcdef1234567890'
        "#;
        assert_eq!(
            config.extract_api_key_from_config(content),
            Some("sk-abcdef1234567890".to_string())
        );

        // Empty case
        let content = r#"
        [openai]
        openai_api_key = ""
        "#;
        assert_eq!(config.extract_api_key_from_config(content), None);
    }

    #[test]
    fn test_parse_providers_config() {
        let mut config = LlmConfig::default();

        let content = r#"
        [providers]

        [providers.openai]
        api_key = "sk-openai1234567890"
        model = "gpt-4"
        is_active = true

        [providers.gemini]
        api_key = "gemini-key1234567890"
        model = "gemini-pro"
        is_active = false
        "#;

        config.parse_providers_config(content);

        // Check OpenAI provider
        assert!(config.providers.contains_key("openai"));
        let openai = config.providers.get("openai").unwrap();
        assert_eq!(openai.api_key, "sk-openai1234567890");
        assert_eq!(openai.model, "gpt-4");
        assert!(openai.is_active);

        // Check Gemini provider
        assert!(config.providers.contains_key("gemini"));
        let gemini = config.providers.get("gemini").unwrap();
        assert_eq!(gemini.api_key, "gemini-key1234567890");
        assert_eq!(gemini.model, "gemini-pro");
        assert!(!gemini.is_active);

        // Check that the active provider is updated
        assert_eq!(config.provider, LlmProvider::OpenAI);
        assert_eq!(config.api_key, "sk-openai1234567890");
        assert_eq!(config.model, "gpt-4");
    }
}
