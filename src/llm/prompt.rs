use std::collections::HashMap;
use std::env;

/// Template for constructing prompts to send to LLM API
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    system_template: String,
    user_template: String,
    variables: HashMap<String, String>,
}

impl PromptTemplate {
    /// Create a new PromptTemplate with default templates
    pub fn new() -> Self {
        let system_template = r#"You are CommandGPT, a specialized AI assistant for generating command-line commands from natural language instructions.

Your task is to convert the user's natural language request into the most appropriate command-line command.

Follow these rules:
1. Generate shell commands for the user's operating system ({os_info}).
2. Generate a single command when possible, but use pipes or command chaining when necessary.
3. Provide brief explanations for complex commands.
4. Assess the safety level of each command (SAFE, CAUTION, DANGEROUS).
5. If a command is potentially destructive, suggest safer alternatives.
6. If the user's request is ambiguous, provide multiple command options if appropriate.
7. Format your response as valid JSON according to the specified schema.

Do NOT:
- Generate commands that could cause significant data loss without warnings
- Execute any commands yourself
- Include unnecessary verbosity in your explanations

Output JSON Schema:
{
  "commands": [
    {
      "command": "the shell command to execute",
      "description": "brief description of what the command does",
      "safety_level": "SAFE|CAUTION|DANGEROUS",
      "explanation": {
        "purpose": "main purpose of the command",
        "components": [
          {"part": "command part or flag", "explanation": "what this part does"}
        ]
      }
    }
  ],
  "is_ambiguous": boolean,
  "additional_questions": ["question to clarify ambiguity", ...]
}"#.to_string();

        let user_template = r#"I need a {shell_type} command to {user_input}.
OS: {os_info}"#
            .to_string();

        Self {
            system_template,
            user_template,
            variables: HashMap::new(),
        }
    }

    /// Set a variable for template replacement
    pub fn set_variable(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Generate the system message with variables replaced
    pub fn build_system_message(&self) -> String {
        let mut result = self.system_template.clone();
        for (key, value) in &self.variables {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }

    /// Generate the user message with variables replaced
    pub fn build_user_message(&self, user_input: &str) -> String {
        let mut result = self.user_template.clone();

        // Set user input
        result = result.replace("{user_input}", user_input);

        // Replace all other variables
        for (key, value) in &self.variables {
            result = result.replace(&format!("{{{}}}", key), value);
        }

        result
    }

    /// Detect and set OS information
    pub fn detect_and_set_os_info(&mut self) -> &mut Self {
        let os_info = Self::get_os_info();
        self.set_variable("os_info", os_info)
    }

    /// Detect and set shell type
    pub fn detect_and_set_shell_type(&mut self) -> &mut Self {
        let shell_type = Self::get_shell_type();
        self.set_variable("shell_type", shell_type)
    }

    /// Get OS information
    fn get_os_info() -> String {
        #[cfg(target_os = "linux")]
        {
            let release_info = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
            for line in release_info.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    return line
                        .trim_start_matches("PRETTY_NAME=")
                        .trim_matches('"')
                        .to_string();
                }
            }
            "Linux".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .ok();

            if let Some(output) = output {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return format!("macOS {}", version);
                }
            }
            "macOS".to_string()
        }
        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("cmd")
                .args(&["/c", "ver"])
                .output()
                .ok();

            if let Some(output) = output {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return version;
                }
            }
            "Windows".to_string()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            "Unknown OS".to_string()
        }
    }

    /// Get shell type
    fn get_shell_type() -> String {
        if let Ok(shell) = env::var("SHELL") {
            if shell.contains("bash") {
                return "Bash".to_string();
            } else if shell.contains("zsh") {
                return "Zsh".to_string();
            } else if shell.contains("fish") {
                return "Fish".to_string();
            }
        }

        #[cfg(target_os = "windows")]
        {
            return "PowerShell".to_string();
        }

        "Bash".to_string() // Default to Bash if we can't detect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_replacement() {
        let mut template = PromptTemplate::new();
        template.set_variable("os_info", "Ubuntu Linux 20.04");
        template.set_variable("shell_type", "Bash");

        let system_message = template.build_system_message();
        assert!(system_message.contains("Ubuntu Linux 20.04"));

        let user_message = template.build_user_message("list all files in the current directory");
        assert!(user_message
            .contains("I need a Bash command to list all files in the current directory"));
        assert!(user_message.contains("OS: Ubuntu Linux 20.04"));
    }

    #[test]
    fn test_os_and_shell_detection() {
        let mut template = PromptTemplate::new();
        template
            .detect_and_set_os_info()
            .detect_and_set_shell_type();

        let system_message = template.build_system_message();
        let user_message = template.build_user_message("test");

        // Just check that the messages don't have empty placeholders
        assert!(!system_message.contains("{os_info}"));
        assert!(!user_message.contains("{shell_type}"));
    }
}
