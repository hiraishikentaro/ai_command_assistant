// Integration tests for input processing

mod common;

use acia::input::{InputMode, InputProcessor, Language};

#[test]
fn test_input_processor_english() {
    // Test English input
    let input_text = "How do I list all files in the current directory?";
    let result = InputProcessor::process(input_text, InputMode::CommandLine);
    assert!(result.is_ok());

    let user_input = result.unwrap();
    assert_eq!(user_input.raw_text, input_text);
    assert_eq!(user_input.detected_language, Language::English);
    assert_eq!(user_input.input_mode, InputMode::CommandLine);
}

#[test]
fn test_input_processor_japanese() {
    // Test Japanese input
    let input_text = "現在のディレクトリのファイル一覧を表示するには？";
    let result = InputProcessor::process(input_text, InputMode::CommandLine);
    assert!(result.is_ok());

    let user_input = result.unwrap();
    assert_eq!(user_input.raw_text, input_text);
    assert_eq!(user_input.detected_language, Language::Japanese);
    assert_eq!(user_input.input_mode, InputMode::CommandLine);
}

#[test]
fn test_input_processor_normalization() {
    // Test normalization of text with excessive whitespace and punctuation
    let input_text = "  How   do I  list  all  files???  ";
    let result = InputProcessor::process(input_text, InputMode::CommandLine);
    assert!(result.is_ok());

    let user_input = result.unwrap();
    assert_eq!(user_input.raw_text, input_text);
    // Actual normalized text keeps the question marks but normalizes whitespace
    assert_eq!(user_input.normalized_text, "How do I list all files???");
}

#[test]
fn test_input_processor_interactive_mode() {
    // Test interactive mode
    let input_text = "list all hidden files";
    let result = InputProcessor::process(input_text, InputMode::Interactive);
    assert!(result.is_ok());

    let user_input = result.unwrap();
    assert_eq!(user_input.input_mode, InputMode::Interactive);
}

#[test]
fn test_input_processor_empty_input() {
    // Test empty input handling
    let input_text = "";
    let result = InputProcessor::process(input_text, InputMode::CommandLine);
    assert!(result.is_err());
}

#[test]
fn test_input_processor_whitespace_only() {
    // Test whitespace-only input handling
    let input_text = "   \t   \n   ";
    let result = InputProcessor::process(input_text, InputMode::CommandLine);
    assert!(result.is_err());
}
