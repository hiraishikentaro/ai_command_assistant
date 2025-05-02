use thiserror::Error;

/// LLM API関連のエラー定義
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("API key is not set. Please set the ACIA_OPENAI_API_KEY environment variable or configure it in ~/.config/acia/config.toml")]
    ApiKeyNotSet,

    #[error("Failed to connect to LLM API: {0}")]
    ConnectionError(#[from] reqwest::Error),

    #[error("Failed to parse API response: {0}")]
    ResponseParseError(String),

    #[error("API request timeout after {0} seconds")]
    TimeoutError(u64),

    #[error("API authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Unexpected API response: {0}")]
    UnexpectedResponse(String),

    #[error("Command generation failed: {0}")]
    CommandGenerationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
