use acia::cli::{Args, ConfigCommand, InteractiveMode};
use acia::command::{CommandGenerator, CommandSafetyValidator};
use acia::explanation::CommandExplanationFormatter;
use acia::input::{InputMode, InputProcessor};
use anyhow::Result;
use colored::Colorize;
use env_logger;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse_args();

    // Determine which command to run
    if args.is_config_command() {
        // Run config command
        ConfigCommand::run().await?;
    } else if args.is_interactive_mode() {
        // Run in interactive mode
        let mut interactive = InteractiveMode::new()?;
        interactive.start()?;
    } else if let Some(input_text) = args.input.as_deref() {
        // Process input from command line arguments
        process_command_line_input(input_text, &args).await?;
    } else {
        // This shouldn't happen (covered by Args::is_interactive_mode check)
        eprintln!(
            "{}",
            "Error: No arguments provided. See --help for usage information.".bright_red()
        );
        std::process::exit(1);
    }

    Ok(())
}

/// Process input from command line arguments
async fn process_command_line_input(input_text: &str, args: &Args) -> Result<()> {
    println!("Input: {}", input_text);

    // Process the input
    match InputProcessor::process(input_text, InputMode::CommandLine) {
        Ok(user_input) => {
            println!("Normalized input: {}", user_input.normalized_text);

            if args.verbose {
                println!("Detected language: {:?}", user_input.detected_language);
                println!("Input mode: {:?}", user_input.input_mode);
                println!("Timestamp: {}", user_input.timestamp);
            }

            // Create command generator and generate commands
            println!("\n{}", "Generating commands...".bright_yellow());
            match CommandGenerator::new().await {
                Ok(generator) => {
                    match generator.generate(&user_input).await {
                        Ok(mut command_generation) => {
                            // Validate command safety
                            let validator = CommandSafetyValidator::default();
                            validator.validate_all(&mut command_generation.commands);

                            // Display results
                            display_commands(&command_generation, args.verbose);

                            // If needed, handle execution or copying to clipboard
                            if args.execute {
                                // TODO: Implement command execution
                                println!(
                                    "{}",
                                    "Command execution not yet implemented".bright_yellow()
                                );
                            }

                            if args.copy {
                                // TODO: Implement clipboard functionality
                                println!(
                                    "{}",
                                    "Clipboard copy not yet implemented".bright_yellow()
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "{}",
                                format!("Error generating commands: {}", e).bright_red()
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!("Error initializing command generator: {}", e).bright_red()
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Error processing input: {}", e).bright_red());
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Display the generated commands
fn display_commands(generation: &acia::llm::CommandGeneration, verbose: bool) {
    if generation.commands.is_empty() {
        println!("{}", "No commands were generated.".bright_red());
        return;
    }

    // Display ambiguity notice if applicable
    if generation.is_ambiguous {
        println!(
            "\n{}",
            "Your request was ambiguous. Here are possible interpretations:".bright_yellow()
        );

        if !generation.additional_questions.is_empty() {
            println!("\n{}", "You might want to clarify:".bright_cyan());
            for (i, question) in generation.additional_questions.iter().enumerate() {
                println!("  {}. {}", i + 1, question);
            }
            println!();
        }
    }

    // Display each command
    for (i, cmd) in generation.commands.iter().enumerate() {
        if generation.commands.len() > 1 {
            println!(
                "\n{} {}",
                "Option".bright_blue(),
                (i + 1).to_string().bright_blue()
            );
        }

        // Create a formatter for the command explanation
        let formatter = CommandExplanationFormatter::default()
            .with_terminal_width(
                // Try to get the terminal width, default to 80 if not available
                terminal_size::terminal_size()
                    .map(|(width, _)| width.0 as usize)
                    .unwrap_or(80),
            )
            .with_related_commands(verbose);

        // Format and print the explanation
        let formatted = formatter.format(cmd);
        print!("{}", formatted);

        // Additional safety warning for dangerous commands
        if cmd.safety_level == acia::llm::SafetyLevel::Dangerous {
            println!(
                "{}",
                "⚠️  WARNING: This command may cause data loss or system changes! ⚠️".bright_red()
            );
        } else if cmd.safety_level == acia::llm::SafetyLevel::Caution {
            println!(
                "{}",
                "⚠️  Caution: Review this command before execution.".bright_yellow()
            );
        }
    }
}
