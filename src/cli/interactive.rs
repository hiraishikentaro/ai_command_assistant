use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::io;

use crate::input::{InputMode, InputProcessor};

pub struct InteractiveMode {
    editor: DefaultEditor,
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

        Ok(Self { editor })
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
                println!("\nDetected language: {:?}", user_input.detected_language);

                // TODO: Call command generation logic here
                println!(
                    "\n(Command generation and display will be implemented here - coming soon)"
                );
                println!(); // Add empty line for readability
            }
            Err(e) => {
                eprintln!("{}", e.to_string().bright_red());
            }
        }

        Ok(())
    }
}
