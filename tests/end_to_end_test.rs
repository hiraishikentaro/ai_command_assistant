// End-to-end integration tests

mod common;

use acia::command::CommandSafetyValidator;
use acia::input::{InputMode, InputProcessor};
use tokio;

// This test simulates the entire pipeline from input to safety-checked commands
// Note: This test may fail if API keys are not configured
#[tokio::test]
#[ignore] // Marked as ignore to avoid running in CI without API keys
async fn test_end_to_end_flow() {
    // 1. Process user input
    let input_text = "How do I list all files in the current directory?";
    let result = InputProcessor::process(input_text, InputMode::CommandLine);
    assert!(result.is_ok());

    let user_input = result.unwrap();

    // 2. Generate commands (requires API key to be configured)
    // Since we can't rely on API keys in CI, we'll test this conditionally
    match acia::command::CommandGenerator::new().await {
        Ok(generator) => {
            let command_result = generator.generate(&user_input).await;

            if let Ok(mut command_generation) = command_result {
                // 3. Validate command safety
                let validator = CommandSafetyValidator::default();
                validator.validate_all(&mut command_generation.commands);

                // 4. Verify we have some commands
                assert!(!command_generation.commands.is_empty());

                // 5. Check that at least one command contains 'ls' for listing files
                let has_ls_command = command_generation
                    .commands
                    .iter()
                    .any(|cmd| cmd.command.contains("ls"));

                assert!(
                    has_ls_command,
                    "Expected at least one 'ls' command for listing files"
                );
            } else {
                // If API call failed, just log it (don't fail the test)
                println!("API call failed: {:?}", command_result.err());
            }
        }
        Err(e) => {
            // If API keys are not configured, just log it (don't fail the test)
            println!(
                "Skipping command generation test due to configuration error: {:?}",
                e
            );
        }
    }
}

// This test simulates the flow with a potentially dangerous command
#[tokio::test]
#[ignore] // Marked as ignore to avoid running in CI without API keys
async fn test_end_to_end_safety_check() {
    // 1. Process user input for a potentially dangerous operation
    let input_text = "How do I delete all files in the root directory?";
    let result = InputProcessor::process(input_text, InputMode::CommandLine);
    assert!(result.is_ok());

    let user_input = result.unwrap();

    // 2. Generate commands (requires API key to be configured)
    match acia::command::CommandGenerator::new().await {
        Ok(generator) => {
            let command_result = generator.generate(&user_input).await;

            if let Ok(mut command_generation) = command_result {
                // Save the original safety levels
                let original_levels: Vec<_> = command_generation
                    .commands
                    .iter()
                    .map(|cmd| (cmd.command.clone(), cmd.safety_level))
                    .collect();

                // 3. Validate command safety
                let validator = CommandSafetyValidator::default();
                validator.validate_all(&mut command_generation.commands);

                // 4. Verify that dangerous commands are properly flagged
                for (i, (cmd, _original_level)) in original_levels.iter().enumerate() {
                    if cmd.contains("rm -rf /") || cmd.contains("rm -r /") {
                        // This command should be marked as dangerous after validation
                        assert_eq!(
                            command_generation.commands[i].safety_level,
                            acia::llm::SafetyLevel::Dangerous,
                            "Command '{}' should be marked as dangerous",
                            cmd
                        );
                    }
                }
            } else {
                // If API call failed, just log it (don't fail the test)
                println!("API call failed: {:?}", command_result.err());
            }
        }
        Err(e) => {
            // If API keys are not configured, just log it (don't fail the test)
            println!(
                "Skipping command generation test due to configuration error: {:?}",
                e
            );
        }
    }
}
