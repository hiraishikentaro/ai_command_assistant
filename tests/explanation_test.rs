mod common;
use acia::explanation::{CommandExplanationFormatter, ExplanationStyle};
use acia::llm::response::{CommandComponent, CommandExplanation};
use acia::llm::{CommandCandidate, SafetyLevel};
use colored::Colorize;
use std::sync::Arc;

#[test]
fn test_explanation_formatter() {
    // Create a test command candidate
    let candidate = CommandCandidate {
        command: "find . -name \"*.txt\" | xargs wc -l".to_string(),
        description: "Find all text files and count lines".to_string(),
        safety_level: SafetyLevel::Safe,
        explanation: CommandExplanation {
            purpose: "Find all text files in current directory and count their lines".to_string(),
            components: vec![
                CommandComponent {
                    part: "find . -name \"*.txt\"".to_string(),
                    explanation: "Search for all .txt files in current directory".to_string(),
                },
                CommandComponent {
                    part: "xargs wc -l".to_string(),
                    explanation: "Count lines in each found file".to_string(),
                },
            ],
        },
    };

    // Create a formatter
    let formatter = CommandExplanationFormatter::default();

    // Format the explanation
    let formatted = formatter.format(&candidate);

    // Check that the formatted explanation contains the expected components
    assert!(formatted.contains("COMMAND:"));
    assert!(formatted.contains("find . -name \"*.txt\" | xargs wc -l"));
    assert!(formatted.contains("PURPOSE:"));
    assert!(formatted.contains("Find all text files in current directory and count their lines"));
    assert!(formatted.contains("OPTIONS:"));
    assert!(formatted.contains("find . -name \"*.txt\""));
    assert!(formatted.contains("Search for all .txt files in current directory"));
    assert!(formatted.contains("xargs wc -l"));
    assert!(formatted.contains("Count lines in each found file"));
    assert!(formatted.contains("RELATED:"));
}

#[test]
fn test_custom_style() {
    // Create a custom style
    let custom_style = ExplanationStyle {
        heading_style: Arc::new(|text: &str| text.red()),
        command_style: Arc::new(|text: &str, _: &SafetyLevel| text.blue()),
        option_style: Arc::new(|text: &str| text.green()),
        related_style: Arc::new(|text: &str| text.yellow()),
    };

    // Create a formatter with the custom style
    let formatter = CommandExplanationFormatter::new(custom_style);

    // Create a basic command
    let candidate = CommandCandidate {
        command: "ls -la".to_string(),
        description: "List files".to_string(),
        safety_level: SafetyLevel::Safe,
        explanation: CommandExplanation {
            purpose: "List all files including hidden ones".to_string(),
            components: vec![
                CommandComponent {
                    part: "ls".to_string(),
                    explanation: "List directory contents".to_string(),
                },
                CommandComponent {
                    part: "-la".to_string(),
                    explanation: "Show hidden files and details".to_string(),
                },
            ],
        },
    };

    // Format the explanation
    let formatted = formatter.format(&candidate);

    // We can't easily test the colors directly, but we can ensure the content is there
    assert!(formatted.contains("COMMAND:"));
    assert!(formatted.contains("ls -la"));
    assert!(formatted.contains("PURPOSE:"));
    assert!(formatted.contains("OPTIONS:"));
    assert!(formatted.contains("ls"));
    assert!(formatted.contains("-la"));
}

#[test]
fn test_related_commands() {
    // Test with a command that should have related commands
    let find_cmd = CommandCandidate {
        command: "find . -type f -name \"*.log\"".to_string(),
        description: "Find log files".to_string(),
        safety_level: SafetyLevel::Safe,
        explanation: CommandExplanation {
            purpose: "Find all log files".to_string(),
            components: vec![CommandComponent {
                part: "find . -type f -name \"*.log\"".to_string(),
                explanation: "Search for .log files".to_string(),
            }],
        },
    };

    let formatter = CommandExplanationFormatter::default();
    let formatted = formatter.format(&find_cmd);

    // The related commands for find should include ls, grep, locate
    assert!(formatted.contains("RELATED:"));
    assert!(formatted.contains("ls"));
    assert!(formatted.contains("grep"));
    assert!(formatted.contains("locate"));

    // Test with a command that uses pipes
    let piped_cmd = CommandCandidate {
        command: "find . -name \"*.txt\" | grep \"important\" | wc -l".to_string(),
        description: "Count important lines".to_string(),
        safety_level: SafetyLevel::Safe,
        explanation: CommandExplanation {
            purpose: "Count lines containing 'important'".to_string(),
            components: vec![
                CommandComponent {
                    part: "find . -name \"*.txt\"".to_string(),
                    explanation: "Find text files".to_string(),
                },
                CommandComponent {
                    part: "grep \"important\"".to_string(),
                    explanation: "Filter for important".to_string(),
                },
                CommandComponent {
                    part: "wc -l".to_string(),
                    explanation: "Count lines".to_string(),
                },
            ],
        },
    };

    let formatted = formatter.format(&piped_cmd);

    // Should include related commands for both find and grep
    assert!(formatted.contains("RELATED:"));
    assert!(formatted.contains("sort")); // Related to wc
    assert!(formatted.contains("uniq")); // Related to wc
    assert!(formatted.contains("sed")); // Related to grep
    assert!(formatted.contains("awk")); // Related to grep
}
