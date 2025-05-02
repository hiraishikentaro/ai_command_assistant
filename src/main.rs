use acia::cli::{Args, InteractiveMode};
use acia::input::{InputMode, InputProcessor};
use anyhow::Result;
use colored::Colorize;

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse_args();

    // Determine the mode (interactive or command line arguments)
    if args.is_interactive_mode() {
        // Run in interactive mode
        let mut interactive = InteractiveMode::new()?;
        interactive.start()?;
    } else if let Some(input_text) = args.input.as_deref() {
        // Process input from command line arguments
        process_command_line_input(input_text, &args)?;
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
fn process_command_line_input(input_text: &str, args: &Args) -> Result<()> {
    println!("Input: {}", input_text);

    // Process the input
    match InputProcessor::process(input_text, InputMode::CommandLine) {
        Ok(user_input) => {
            println!("Normalized input: {}", user_input.normalized_text);
            println!("Detected language: {:?}", user_input.detected_language);

            if args.verbose {
                println!("Input mode: {:?}", user_input.input_mode);
                println!("Timestamp: {}", user_input.timestamp);
            }

            // TODO: Call command generation logic here
            println!("\n(Command generation and display will be implemented here - coming soon)");
        }
        Err(e) => {
            eprintln!("{}", format!("Error: {}", e).bright_red());
            std::process::exit(1);
        }
    }

    Ok(())
}
