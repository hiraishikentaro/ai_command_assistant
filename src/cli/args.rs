use clap::Parser;

/// AI Command Assistant - CLI assistant that generates shell commands from natural language
#[derive(Parser, Debug)]
#[command(
    name = "acia",
    version,
    about,
    long_about = "AI Command Assistant leverages AI to generate and explain shell commands from natural language instructions."
)]
pub struct Args {
    /// Natural language instruction (omit for interactive mode)
    #[arg(index = 1)]
    pub input: Option<String>,

    /// Execute the generated command after confirmation
    #[arg(short, long)]
    pub execute: bool,

    /// Copy the generated command to clipboard
    #[arg(short, long)]
    pub copy: bool,

    /// Display detailed explanation
    #[arg(short, long)]
    pub verbose: bool,
}

impl Args {
    /// Parse and return command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Check if we should start in interactive mode
    pub fn is_interactive_mode(&self) -> bool {
        self.input.is_none()
    }
}
