# AI Command Assistant (ACliA)

ACliA is a command-line tool that uses AI to convert natural language instructions into shell commands. Simply describe what you want to do, and ACliA will generate the appropriate command for you.

## Features

- Convert natural language instructions to shell commands
- Support for both English and Japanese input
- Interactive mode and command-line mode
- Safety rating for each command (Safe, Caution, Dangerous)
- Detailed explanations of commands and their components
- Multiple command suggestions for ambiguous requests

## Installation

### Prerequisites

- Rust and Cargo (1.70 or later)
- OpenAI API key (required for command generation)

### Building from Source

1. Clone the repository:

   ```
   git clone https://github.com/yourusername/ai_command_assistant.git
   cd ai_command_assistant
   ```

2. Build the project:

   ```
   cargo build --release
   ```

3. The compiled binary will be available at `target/release/acia`.

### Configuration

Before using ACliA, you need to configure your OpenAI API key. You can do this in two ways:

1. Using the setup script:

   ```
   ./setup.sh
   ```

2. Manually creating a configuration file at `~/.config/acia/config.toml`:

   ```toml
   # OpenAI API Key
   openai_api_key = "your-api-key-here"

   # LLM Settings
   model = "gpt-3.5-turbo"
   temperature = 0.2
   max_tokens = 500
   timeout_seconds = 30
   ```

3. Setting an environment variable:
   ```
   export ACIA_OPENAI_API_KEY="your-api-key-here"
   ```

## Usage

### Interactive Mode

Run ACliA in interactive mode:

```
acia
```

In interactive mode, you can type natural language instructions, and ACliA will generate the appropriate commands.

### Command-Line Mode

Generate a command from a natural language instruction:

```
acia "list all files in the current directory"
```

### Options

- `-v, --verbose`: Display detailed explanations of commands
- `-e, --execute`: Execute the generated command after confirmation (not yet implemented)
- `-c, --copy`: Copy the generated command to clipboard (not yet implemented)

## Examples

```
$ acia "find all PDF files in the current directory and subdirectories"

Command: find . -name "*.pdf"
Safety: SAFE
Description: Search for PDF files in the current directory and its subdirectories

$ acia "create a backup of my documents folder that's compressed"

Command: tar -czvf documents_backup.tar.gz ~/Documents
Safety: SAFE
Description: Create a compressed archive of the Documents folder
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
