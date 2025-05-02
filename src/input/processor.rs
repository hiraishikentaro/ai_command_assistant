use chrono::{DateTime, Utc};
use std::io;

use super::language_detector::{Language, LanguageDetector};
use super::normalizer::TextNormalizer;

/// Input mode (command line arguments or interactive mode)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    /// Input from command line arguments
    CommandLine,
    /// Input from interactive prompt
    Interactive,
}

/// User input information
#[derive(Debug, Clone)]
pub struct UserInput {
    /// Original input text
    pub raw_text: String,
    /// Normalized text
    pub normalized_text: String,
    /// Detected language
    pub detected_language: Language,
    /// Input mode (CLI args/interactive)
    pub input_mode: InputMode,
    /// Input timestamp
    pub timestamp: DateTime<Utc>,
}

/// Input processing class
pub struct InputProcessor;

impl InputProcessor {
    /// Process input text
    pub fn process(text: &str, mode: InputMode) -> Result<UserInput, io::Error> {
        // Check for empty input
        if text.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Input is empty. Please ask a question about commands.",
            ));
        }

        // Normalize text
        let normalized_text = TextNormalizer::normalize(text);

        // Detect language
        let detected_language = LanguageDetector::detect(&normalized_text);

        // Get current timestamp
        let timestamp = Utc::now();

        // Create UserInput object
        let user_input = UserInput {
            raw_text: text.to_string(),
            normalized_text,
            detected_language,
            input_mode: mode,
            timestamp,
        };

        Ok(user_input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_valid_input() {
        let result = InputProcessor::process("ファイルを検索", InputMode::CommandLine);
        assert!(result.is_ok());

        let input = result.unwrap();
        assert_eq!(input.raw_text, "ファイルを検索");
        assert_eq!(input.normalized_text, "ファイルを検索");
        assert_eq!(input.detected_language, Language::Japanese);
        assert_eq!(input.input_mode, InputMode::CommandLine);
    }

    #[test]
    fn test_process_normalized_input() {
        let result = InputProcessor::process("  ファイル　を  検索  ", InputMode::Interactive);
        assert!(result.is_ok());

        let input = result.unwrap();
        assert_eq!(input.raw_text, "  ファイル　を  検索  ");
        assert_eq!(input.normalized_text, "ファイル を 検索"); // Whitespace normalization
        assert_eq!(input.detected_language, Language::Japanese);
        assert_eq!(input.input_mode, InputMode::Interactive);
    }

    #[test]
    fn test_process_empty_input() {
        let result = InputProcessor::process("", InputMode::CommandLine);
        assert!(result.is_err());

        let result = InputProcessor::process("   ", InputMode::CommandLine);
        assert!(result.is_err());
    }
}
