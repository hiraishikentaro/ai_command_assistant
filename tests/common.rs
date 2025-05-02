// Common test utilities for integration tests

use acia::input::{InputMode, UserInput};
use acia::llm::response::{CommandComponent, CommandExplanation};
use acia::llm::{CommandCandidate, CommandGeneration, SafetyLevel};
use chrono::Utc;

/// Create a test user input
#[allow(dead_code)]
pub fn create_test_input(text: &str) -> UserInput {
    UserInput {
        raw_text: text.to_string(),
        normalized_text: text.to_string(),
        detected_language: acia::input::Language::English,
        input_mode: InputMode::CommandLine,
        timestamp: Utc::now(),
    }
}

/// Create a mock command generation result
#[allow(dead_code)]
pub fn create_test_command_generation(commands: Vec<&str>) -> CommandGeneration {
    let command_candidates = commands
        .into_iter()
        .map(|cmd| CommandCandidate {
            command: cmd.to_string(),
            description: format!("Description for {}", cmd),
            safety_level: SafetyLevel::Safe,
            explanation: CommandExplanation {
                purpose: format!("Purpose of {}", cmd),
                components: vec![CommandComponent {
                    part: cmd.to_string(),
                    explanation: format!("Explanation of {}", cmd),
                }],
            },
        })
        .collect();

    CommandGeneration {
        commands: command_candidates,
        is_ambiguous: false,
        additional_questions: vec![],
        raw_response: "".to_string(),
    }
}

/// Create a test command with specified safety level
#[allow(dead_code)]
pub fn create_test_command(cmd: &str, safety: SafetyLevel) -> CommandCandidate {
    CommandCandidate {
        command: cmd.to_string(),
        description: format!("Description for {}", cmd),
        safety_level: safety,
        explanation: CommandExplanation {
            purpose: format!("Purpose of {}", cmd),
            components: vec![CommandComponent {
                part: cmd.to_string(),
                explanation: format!("Explanation of {}", cmd),
            }],
        },
    }
}
