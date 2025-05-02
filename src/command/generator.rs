use std::sync::Arc;

use crate::input::{InputMode, InputProcessor, UserInput};
use crate::llm::client::LlmClientFactory;
use crate::llm::{CommandGeneration, LlmClient, LlmConfig, LlmError};

/// Command generator for converting natural language to shell commands
pub struct CommandGenerator {
    llm_client: Arc<Box<dyn LlmClient>>,
}

impl CommandGenerator {
    /// Create a new command generator with default configuration
    pub async fn new() -> Result<Self, LlmError> {
        // Load the configuration from file instead of using the default
        let config = match LlmConfig::load() {
            Ok(config) => config,
            Err(e) => {
                log::warn!("Failed to load LLM config, using default: {}", e);
                LlmConfig::new()
            }
        };
        Self::with_config(config).await
    }

    /// Create a new command generator with custom configuration
    pub async fn with_config(config: LlmConfig) -> Result<Self, LlmError> {
        let llm_client = Arc::new(LlmClientFactory::create_client(config)?);
        Ok(Self { llm_client })
    }

    /// Generate commands from user input
    pub async fn generate(&self, input: &UserInput) -> Result<CommandGeneration, LlmError> {
        // Use the normalized text from input processing
        self.llm_client
            .generate_commands(&input.normalized_text)
            .await
    }

    /// Generate commands directly from text
    pub async fn generate_from_text(&self, text: &str) -> Result<CommandGeneration, LlmError> {
        // Process the input first
        let input = InputProcessor::process(text, InputMode::CommandLine)?;
        self.generate(&input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::SafetyLevel;
    use crate::llm::response::{CommandCandidate, CommandComponent, CommandExplanation};
    use mockall::predicate::*;
    use mockall::*;

    // Mock the LlmClient trait
    mock! {
        pub LlmClient {}

        #[async_trait::async_trait]
        impl LlmClient for LlmClient {
            async fn generate_commands(&self, input: &str) -> Result<CommandGeneration, LlmError>;
        }
    }

    #[tokio::test]
    async fn test_generate_commands() {
        // Create mock response
        let command_gen = CommandGeneration {
            commands: vec![CommandCandidate {
                command: "ls -la".to_string(),
                description: "List all files".to_string(),
                safety_level: SafetyLevel::Safe,
                explanation: CommandExplanation {
                    purpose: "List files".to_string(),
                    components: vec![CommandComponent {
                        part: "ls".to_string(),
                        explanation: "List directory contents".to_string(),
                    }],
                },
            }],
            is_ambiguous: false,
            additional_questions: vec![],
            raw_response: "".to_string(),
        };

        // Create mock LLM client
        let mut mock_client = MockLlmClient::new();
        mock_client
            .expect_generate_commands()
            .with(eq("list all files"))
            .times(1)
            .returning(move |_| Ok(command_gen.clone()));

        // Create command generator with mock client
        let generator = CommandGenerator {
            llm_client: Arc::new(Box::new(mock_client) as Box<dyn LlmClient>),
        };

        // Create test input
        let input = UserInput {
            raw_text: "list all files".to_string(),
            normalized_text: "list all files".to_string(),
            detected_language: crate::input::Language::English,
            input_mode: crate::input::InputMode::CommandLine,
            timestamp: chrono::Utc::now(),
        };

        // Test command generation
        let result = generator.generate(&input).await;
        assert!(result.is_ok());

        let command_gen = result.unwrap();
        assert_eq!(command_gen.commands.len(), 1);
        assert_eq!(command_gen.commands[0].command, "ls -la");
    }
}
