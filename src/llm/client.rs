use async_trait::async_trait;
use reqwest::{header, Client as HttpClient};
use std::time::Duration;
use tokio::time::timeout;

use crate::llm::config::LlmConfig;
use crate::llm::error::LlmError;
use crate::llm::prompt::PromptTemplate;
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

/// Factory for creating appropriate LLM clients
pub struct LlmClientFactory;

impl LlmClientFactory {
    /// Create a new LLM client based on configuration
    pub fn create_client(config: LlmConfig) -> Result<Box<dyn LlmClient>, LlmError> {
        // Make sure API key is loaded
        let mut config = config;
        config.load_api_key()?;

        // For now, we only support OpenAI client
        let client = OpenAiClient::new(config)?;
        Ok(Box::new(client))
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
