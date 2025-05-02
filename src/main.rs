use acia::cli::{Args, ConfigCommand, InteractiveMode};
use acia::command::{CommandGenerator, CommandSafetyValidator};
use acia::input::{InputMode, InputProcessor};
use anyhow::Result;
use colored::Colorize;
use env_logger;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
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

        // Display command with appropriate color based on safety
        let command_display = match cmd.safety_level {
            acia::llm::SafetyLevel::Safe => cmd.command.bright_green(),
            acia::llm::SafetyLevel::Caution => cmd.command.bright_yellow(),
            acia::llm::SafetyLevel::Dangerous => cmd.command.bright_red(),
        };

        println!("\n{} {}", "Command:".bright_blue(), command_display);

        // Display safety level
        let safety_display = match cmd.safety_level {
            acia::llm::SafetyLevel::Safe => "SAFE".bright_green(),
            acia::llm::SafetyLevel::Caution => "CAUTION".bright_yellow(),
            acia::llm::SafetyLevel::Dangerous => "DANGEROUS".bright_red(),
        };
        println!("{} {}", "Safety:".bright_blue(), safety_display);

        // Display description
        println!("{} {}", "Description:".bright_blue(), cmd.description);

        // Display detailed explanation if verbose
        if verbose {
            println!("\n{} {}", "Purpose:".bright_blue(), cmd.explanation.purpose);

            if !cmd.explanation.components.is_empty() {
                println!("\n{}", "Components:".bright_blue());
                for component in &cmd.explanation.components {
                    println!(
                        "  {} - {}",
                        component.part.bright_cyan(),
                        component.explanation
                    );
                }
            }
        }
    }
}
