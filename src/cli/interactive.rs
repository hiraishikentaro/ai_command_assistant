use colored::Colorize;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::io;
use tokio::runtime::Runtime;

use crate::command::{CommandGenerator, CommandSafetyValidator};
use crate::input::{InputMode, InputProcessor};

pub struct InteractiveMode {
    editor: DefaultEditor,
    runtime: Runtime,
    command_generator: Option<CommandGenerator>,
}

impl InteractiveMode {
    /// Create a new interactive mode instance
    pub fn new() -> Result<Self, io::Error> {
        let editor = DefaultEditor::new().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Readline initialization error: {}", e),
            )
        })?;

        // Create a tokio runtime for async operations
        let runtime = Runtime::new().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create runtime: {}", e),
            )
        })?;

        // Initialize the command generator
        let command_generator = runtime.block_on(async {
            match CommandGenerator::new().await {
                Ok(generator) => Some(generator),
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!("Warning: Could not initialize command generator: {}", e)
                            .bright_yellow()
                    );
                    eprintln!(
                        "{}",
                        "Commands will not be available until API key is configured."
                            .bright_yellow()
                    );
                    None
                }
            }
        });

        Ok(Self {
            editor,
            runtime,
            command_generator,
        })
    }

    /// Start the interactive mode
    pub fn start(&mut self) -> Result<(), io::Error> {
        println!("ACliA v0.1.0");
        println!(
            "Ask about commands in natural language. Type 'exit' or 'quit' to end the session."
        );
        println!();

        loop {
            match self.editor.readline("> ") {
                Ok(line) => {
                    let input = line.trim();

                    // Add to history (if not empty)
                    if !input.is_empty() {
                        let _ = self.editor.add_history_entry(input);
                    }

                    // Handle exit commands
                    if input == "exit" || input == "quit" {
                        println!("Exiting.");
                        break;
                    }

                    // Handle empty input
                    if input.is_empty() {
                        println!("Please ask a question about commands.");
                        continue;
                    }

                    // Process the input
                    self.process_input(input)?;
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Interrupted. Type 'exit' to quit.");
                }
                Err(ReadlineError::Eof) => {
                    println!("Exiting.");
                    break;
                }
                Err(err) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Input error: {}", err),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Process the user input
    fn process_input(&self, input: &str) -> Result<(), io::Error> {
        // Use the input processor module to process the input
        match InputProcessor::process(input, InputMode::Interactive) {
            Ok(user_input) => {
                if self.command_generator.is_none() {
                    eprintln!(
                        "{}",
                        "Command generator is not available. Please configure your API key."
                            .bright_yellow()
                    );
                    return Ok(());
                }

                // Generate commands using the generator
                self.runtime.block_on(async {
                    println!("\n{}", "Generating commands...".bright_yellow());

                    match self
                        .command_generator
                        .as_ref()
                        .unwrap()
                        .generate(&user_input)
                        .await
                    {
                        Ok(mut command_generation) => {
                            // Validate command safety
                            let validator = CommandSafetyValidator::default();
                            validator.validate_all(&mut command_generation.commands);

                            // Display results
                            display_commands(&command_generation, true); // Always show verbose output in interactive mode
                        }
                        Err(e) => {
                            eprintln!(
                                "{}",
                                format!("Error generating commands: {}", e).bright_red()
                            );
                        }
                    }
                });

                println!(); // Add empty line for readability
            }
            Err(e) => {
                eprintln!("{}", e.to_string().bright_red());
            }
        }

        Ok(())
    }
}

/// Display the generated commands
fn display_commands(generation: &crate::llm::CommandGeneration, verbose: bool) {
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
        let formatter = crate::explanation::CommandExplanationFormatter::default()
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
        if cmd.safety_level == crate::llm::SafetyLevel::Dangerous {
            println!(
                "{}",
                "⚠️  WARNING: This command may cause data loss or system changes! ⚠️".bright_red()
            );
        } else if cmd.safety_level == crate::llm::SafetyLevel::Caution {
            println!(
                "{}",
                "⚠️  Caution: Review this command before execution.".bright_yellow()
            );
        }
    }
}
