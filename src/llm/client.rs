use async_trait::async_trait;
use reqwest::{Client as HttpClient, header};
use std::time::Duration;
use tokio::time::timeout;

use crate::llm::config::LlmConfig;
use crate::llm::error::LlmError;
use crate::llm::prompt::PromptTemplate;
use crate::llm::providers::LlmProvider;
use crate::llm::request::OpenAiRequest;
use crate::llm::response::{CommandGeneration, OpenAiResponse};

/// Trait for LLM clients
#[async_trait]
pub trait LlmClient {
    /// Generate commands from a natural language input
    async fn generate_commands(&self, input: &str) -> Result<CommandGeneration, LlmError>;
}

/// OpenAI API client
pub struct OpenAiClient {
    config: LlmConfig,
    http_client: HttpClient,
    prompt_template: PromptTemplate,
}

impl OpenAiClient {
    /// Create a new OpenAI client
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        // Make sure API key is loaded
        if config.api_key.is_empty() {
            return Err(LlmError::ApiKeyNotSet);
        }

        // Create HTTP client with authorization header
        let mut headers = header::HeaderMap::new();
        let auth_value = format!("Bearer {}", config.api_key);
        let mut auth_header = header::HeaderValue::from_str(&auth_value)
            .map_err(|_| LlmError::ConfigError("Invalid API key format".to_string()))?;
        auth_header.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_header);

        let http_client = HttpClient::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| LlmError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        // Create and set up prompt template
        let mut prompt_template = PromptTemplate::new();
        prompt_template
            .detect_and_set_os_info()
            .detect_and_set_shell_type();

        Ok(Self {
            config,
            http_client,
            prompt_template,
        })
    }

    /// Build request with system and user messages
    fn build_request(&self, input: &str) -> OpenAiRequest {
        let system_message = self.prompt_template.build_system_message();
        let user_message = self.prompt_template.build_user_message(input);

        let mut request = OpenAiRequest::new(&self.config.model);
        request
            .with_system_message(system_message)
            .with_user_message(user_message)
            .with_temperature(self.config.temperature)
            .with_max_tokens(self.config.max_tokens);

        request
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn generate_commands(&self, input: &str) -> Result<CommandGeneration, LlmError> {
        let request = self.build_request(input);

        // Make API request with timeout
        let api_url = "https://api.openai.com/v1/chat/completions";
        let request_timeout = Duration::from_secs(self.config.timeout_seconds);

        log::debug!("Sending request to OpenAI API: {}", api_url);

        let response = match timeout(
            request_timeout,
            self.http_client.post(api_url).json(&request).send(),
        )
        .await
        {
            Ok(result) => match result {
                Ok(response) => response,
                Err(e) => return Err(LlmError::ConnectionError(e)),
            },
            Err(_) => {
                return Err(LlmError::TimeoutError(self.config.timeout_seconds));
            }
        };

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 => Err(LlmError::AuthenticationError(error_text)),
                429 => Err(LlmError::RateLimitError(error_text)),
                _ => Err(LlmError::UnexpectedResponse(format!(
                    "API returned status {}: {}",
                    status, error_text
                ))),
            };
        }

        // Parse response
        let openai_response: OpenAiResponse = response.json().await.map_err(|e| {
            LlmError::ResponseParseError(format!("Failed to parse response: {}", e))
        })?;

        // Convert to CommandGeneration
        CommandGeneration::from_openai_response(&openai_response)
    }
}

/// Anthropic Claude API client
pub struct ClaudeClient {
    config: LlmConfig,
    http_client: HttpClient,
    prompt_template: PromptTemplate,
}

impl ClaudeClient {
    /// Create a new Claude client
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        // Make sure API key is loaded
        if config.api_key.is_empty() {
            return Err(LlmError::ApiKeyNotSet);
        }

        // Create HTTP client with authorization header
        let mut headers = header::HeaderMap::new();
        let mut auth_header = header::HeaderValue::from_str(&format!("{}", config.api_key))
            .map_err(|_| LlmError::ConfigError("Invalid API key format".to_string()))?;
        auth_header.set_sensitive(true);
        headers.insert("x-api-key", auth_header);

        // Add required headers for Anthropic API
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        // Add Anthropic API version header
        headers.insert(
            "anthropic-version",
            header::HeaderValue::from_static("2023-06-01"),
        );

        let http_client = HttpClient::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| LlmError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        // Create and set up prompt template
        let mut prompt_template = PromptTemplate::new();
        prompt_template
            .detect_and_set_os_info()
            .detect_and_set_shell_type();

        Ok(Self {
            config,
            http_client,
            prompt_template,
        })
    }

    /// Build Claude API request
    fn build_request(&self, input: &str) -> serde_json::Value {
        let system_message = self.prompt_template.build_system_message();
        let user_message = self.prompt_template.build_user_message(input);

        // Update the model name to use the canonical format if needed
        // Claude model names need to have @YYYYMMDD suffix for proper versioning
        let model_name = if !self.config.model.contains('@') {
            match self.config.model.as_str() {
                "claude-3-haiku" => "claude-3-haiku-20240307",
                "claude-3-sonnet" => "claude-3-sonnet-20240229",
                "claude-3-opus" => "claude-3-opus-20240229",
                "claude-3-5-sonnet" | "claude-3.5-sonnet" => "claude-3-5-sonnet-20240620",
                "claude-3-5-haiku" | "claude-3.5-haiku" => "claude-3-5-haiku-20241022",
                "claude-3-7-sonnet" | "claude-3.7-sonnet" => "claude-3-7-sonnet-20250219",
                _ => &self.config.model, // Use as is if we don't know how to format it
            }
            .to_string()
        } else {
            self.config.model.clone()
        };

        // Format the request according to the Anthropic API specification
        serde_json::json!({
            "model": model_name,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "system": system_message,
            "messages": [
                {
                    "role": "user",
                    "content": user_message
                }
            ]
        })
    }
}

#[async_trait]
impl LlmClient for ClaudeClient {
    async fn generate_commands(&self, input: &str) -> Result<CommandGeneration, LlmError> {
        let request = self.build_request(input);

        // Make API request with timeout
        let api_url = "https://api.anthropic.com/v1/messages";
        let request_timeout = Duration::from_secs(self.config.timeout_seconds);

        let response = match timeout(
            request_timeout,
            self.http_client.post(api_url).json(&request).send(),
        )
        .await
        {
            Ok(result) => match result {
                Ok(response) => response,
                Err(e) => return Err(LlmError::ConnectionError(e)),
            },
            Err(_) => {
                return Err(LlmError::TimeoutError(self.config.timeout_seconds));
            }
        };

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 => Err(LlmError::AuthenticationError(error_text)),
                429 => Err(LlmError::RateLimitError(error_text)),
                _ => Err(LlmError::UnexpectedResponse(format!(
                    "API returned status {}: {}",
                    status, error_text
                ))),
            };
        }

        // Parse response
        let response_text = response.text().await.map_err(|e| {
            LlmError::ResponseParseError(format!("Failed to get response text: {}", e))
        })?;

        let response_json: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| {
                LlmError::ResponseParseError(format!("Failed to parse response as JSON: {}", e))
            })?;

        // Extract content from Claude's response
        if let Some(content) = response_json["content"].as_array() {
            for item in content {
                if let Some(text) = item["text"].as_str() {
                    return CommandGeneration::from_json_str(text);
                }
            }
        }

        Err(LlmError::ResponseParseError(
            "Could not find content in Claude response".to_string(),
        ))
    }
}

/// Google Gemini API client
pub struct GeminiClient {
    config: LlmConfig,
    http_client: HttpClient,
    prompt_template: PromptTemplate,
}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        // Make sure API key is loaded
        if config.api_key.is_empty() {
            return Err(LlmError::ApiKeyNotSet);
        }

        // Create HTTP client (Gemini uses API key in the URL, not in headers)
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| LlmError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        // Create and set up prompt template
        let mut prompt_template = PromptTemplate::new();
        prompt_template
            .detect_and_set_os_info()
            .detect_and_set_shell_type();

        Ok(Self {
            config,
            http_client,
            prompt_template,
        })
    }

    /// Build Gemini API request
    fn build_request(&self, input: &str) -> serde_json::Value {
        let system_message = self.prompt_template.build_system_message();
        let user_message = self.prompt_template.build_user_message(input);

        serde_json::json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        { "text": format!("System instructions: {}\n\nUser request: {}", system_message, user_message) }
                    ]
                }
            ],
            "generationConfig": {
                "temperature": self.config.temperature,
                "maxOutputTokens": self.config.max_tokens,
                "responseMimeType": "application/json"
            }
        })
    }
}

#[async_trait]
impl LlmClient for GeminiClient {
    async fn generate_commands(&self, input: &str) -> Result<CommandGeneration, LlmError> {
        let request = self.build_request(input);

        // Gemini API requires the API key as part of the URL
        let api_url = format!(
            "https://generativelanguage.googleapis.com/v1/models/{}:generateContent?key={}",
            self.config.model, self.config.api_key
        );
        let request_timeout = Duration::from_secs(self.config.timeout_seconds);

        log::debug!(
            "Sending request to Gemini API: {}",
            api_url.split('?').next().unwrap_or(&api_url)
        );

        let response = match timeout(
            request_timeout,
            self.http_client.post(&api_url).json(&request).send(),
        )
        .await
        {
            Ok(result) => match result {
                Ok(response) => response,
                Err(e) => return Err(LlmError::ConnectionError(e)),
            },
            Err(_) => {
                return Err(LlmError::TimeoutError(self.config.timeout_seconds));
            }
        };

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 => Err(LlmError::AuthenticationError(error_text)),
                429 => Err(LlmError::RateLimitError(error_text)),
                _ => Err(LlmError::UnexpectedResponse(format!(
                    "API returned status {}: {}",
                    status, error_text
                ))),
            };
        }

        // Parse response
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            LlmError::ResponseParseError(format!("Failed to parse response: {}", e))
        })?;

        // Extract content from Gemini's response
        if let Some(candidates) = response_json["candidates"].as_array() {
            if let Some(first_candidate) = candidates.first() {
                if let Some(parts) = first_candidate["content"]["parts"].as_array() {
                    if let Some(first_part) = parts.first() {
                        if let Some(text) = first_part["text"].as_str() {
                            return CommandGeneration::from_json_str(text);
                        }
                    }
                }
            }
        }

        Err(LlmError::ResponseParseError(
            "Could not find content in Gemini response".to_string(),
        ))
    }
}

/// Factory for creating appropriate LLM clients
pub struct LlmClientFactory;

impl LlmClientFactory {
    /// Create a new LLM client based on configuration
    pub fn create_client(config: LlmConfig) -> Result<Box<dyn LlmClient>, LlmError> {
        // Make sure API key is loaded
        let mut config = config;
        config.load_api_key()?;

        // Create the appropriate client based on the provider
        match config.provider {
            LlmProvider::OpenAI => {
                let client = OpenAiClient::new(config)?;
                Ok(Box::new(client))
            }
            LlmProvider::Claude => {
                let client = ClaudeClient::new(config)?;
                Ok(Box::new(client))
            }
            LlmProvider::Gemini => {
                let client = GeminiClient::new(config)?;
                Ok(Box::new(client))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        pub HttpClient {}

        #[async_trait]
        impl LlmClient for HttpClient {
            async fn generate_commands(&self, input: &str) -> Result<CommandGeneration, LlmError>;
        }
    }

    #[test]
    fn test_client_creation_without_api_key() {
        let config = LlmConfig::default(); // Empty API key
        let client = OpenAiClient::new(config);
        assert!(matches!(client, Err(LlmError::ApiKeyNotSet)));
    }

    #[test]
    fn test_client_creation_with_api_key() {
        let config = LlmConfig::default().with_model("gpt-4");
        let mut config = config;
        config.api_key = "sk-test123456789".to_string();

        let client = OpenAiClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_request() {
        let mut config = LlmConfig::default();
        config.api_key = "sk-test123456789".to_string();
        config.model = "gpt-4".to_string();
        config.temperature = 0.7;

        let client = OpenAiClient::new(config).unwrap();
        let request = client.build_request("list all files");

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.temperature, 0.7);
        assert_eq!(request.messages.len(), 2);
        assert!(request.messages[1].content.contains("list all files"));
    }
}
