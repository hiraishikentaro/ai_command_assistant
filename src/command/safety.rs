use regex::Regex;
use std::collections::HashSet;

use crate::llm::{CommandCandidate, SafetyLevel};

/// Command safety validator to check if commands are safe to execute
pub struct CommandSafetyValidator {
    dangerous_patterns: HashSet<String>,
    caution_patterns: HashSet<String>,
}

impl Default for CommandSafetyValidator {
    fn default() -> Self {
        let mut validator = Self::new();
        validator.add_default_patterns();
        validator
    }
}

impl CommandSafetyValidator {
    /// Create a new command safety validator
    pub fn new() -> Self {
        Self {
            dangerous_patterns: HashSet::new(),
            caution_patterns: HashSet::new(),
        }
    }

    /// Add default dangerous and caution patterns
    fn add_default_patterns(&mut self) {
        // Dangerous patterns - commands that could cause data loss or system damage
        self.add_dangerous_patterns(&[
            r"rm\s+(-r|-rf|--recursive)\s+/",
            r"rm\s+(-r|-rf|--recursive)\s+~/",
            r"rm\s+(-r|-rf|--recursive)\s+\*/",
            r"rm\s+(-r|-rf|--recursive)\s+\.",
            r"dd\s+.*of=/dev/(disk|hd|sd|nvme)",
            r"mkfs\s+.*\s+/dev/(disk|hd|sd|nvme)",
            r"mv\s+.*\s+/dev/null",
            r":\(\)\{\s*:\|\:&\s*\};:", // Fork bomb
            r"sudo\s+rm\s+(-r|-rf|--recursive)",
            r"chmod\s+777\s+/",
            r"chmod\s+777\s+\*/",
            r"chmod\s+777\s+~/",
            r"chmod\s+777\s+\.",
            r"shutdown",
            r"halt",
            r"reboot",
            r"poweroff",
            r">.*/(passwd|shadow|group)",
            r">\s+/boot",
            r">\s+/etc",
        ]);

        // Caution patterns - commands that should be used with care
        self.add_caution_patterns(&[
            r"rm\s+(-r|-rf|--recursive)",
            r"find\s+.*\s+-delete",
            r"chmod\s+777",
            r"chown\s+.*\s+/",
            r"sudo",
            r"su\s+",
            r"dd\s+",
            r"fdisk",
            r"mkfs",
            r"mkswap",
            r"mount",
            r"umount",
            r"apt(-get)?\s+(remove|purge)",
            r"yum\s+remove",
            r"pacman\s+-R",
            r"brew\s+uninstall",
            r"pip\s+uninstall",
            r"npm\s+uninstall\s+-g",
            r"wget\s+.*\s+\|\s+bash",
            r"curl\s+.*\s+\|\s+bash",
            r">\s+.*\/(config|conf)",
            r"mv\s+.*/(config|conf)",
        ]);
    }

    /// Add custom dangerous patterns
    pub fn add_dangerous_patterns(&mut self, patterns: &[&str]) {
        for pattern in patterns {
            self.dangerous_patterns.insert(pattern.to_string());
        }
    }

    /// Add custom caution patterns
    pub fn add_caution_patterns(&mut self, patterns: &[&str]) {
        for pattern in patterns {
            self.caution_patterns.insert(pattern.to_string());
        }
    }

    /// Validate a command candidate
    pub fn validate(&self, command: &mut CommandCandidate) {
        let new_safety_level = self.check_safety_level(&command.command);

        // If our check determined a more severe safety level than the LLM, update it
        if new_safety_level as u8 > command.safety_level as u8 {
            command.safety_level = new_safety_level;
        }
    }

    /// Validate a vector of command candidates
    pub fn validate_all(&self, commands: &mut Vec<CommandCandidate>) {
        for command in commands {
            self.validate(command);
        }
    }

    /// Check the safety level of a command
    pub fn check_safety_level(&self, command: &str) -> SafetyLevel {
        // Check against dangerous patterns
        for pattern in &self.dangerous_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(command) {
                    return SafetyLevel::Dangerous;
                }
            }
        }

        // Check against caution patterns
        for pattern in &self.caution_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(command) {
                    return SafetyLevel::Caution;
                }
            }
        }

        // If no patterns matched, the command is safe
        SafetyLevel::Safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::response::{CommandCandidate, CommandComponent, CommandExplanation};

    fn create_test_command(cmd: &str) -> CommandCandidate {
        CommandCandidate {
            command: cmd.to_string(),
            description: "Test command".to_string(),
            safety_level: SafetyLevel::Safe, // Start with Safe
            explanation: CommandExplanation {
                purpose: "Testing".to_string(),
                components: vec![CommandComponent {
                    part: cmd.to_string(),
                    explanation: "Test part".to_string(),
                }],
            },
        }
    }

    #[test]
    fn test_safe_commands() {
        let validator = CommandSafetyValidator::default();
        let safe_commands = [
            "ls -la",
            "cd /tmp",
            "echo 'hello world'",
            "grep 'pattern' file.txt",
            "ps aux",
            "mkdir test_dir",
        ];

        for cmd in &safe_commands {
            assert_eq!(validator.check_safety_level(cmd), SafetyLevel::Safe);
        }
    }

    #[test]
    fn test_caution_commands() {
        let validator = CommandSafetyValidator::default();
        let caution_commands = [
            "sudo apt update",
            "chmod 777 myfile.txt",
            "sudo service apache2 restart",
            "find . -name '*.txt' -delete",
            "dd if=/dev/zero of=test bs=1M count=10",
        ];

        for cmd in &caution_commands {
            assert_eq!(validator.check_safety_level(cmd), SafetyLevel::Caution);
        }
    }

    #[test]
    fn test_dangerous_commands() {
        let validator = CommandSafetyValidator::default();
        let dangerous_commands = [
            "rm -rf /",
            "sudo rm -rf ~/",
            "dd if=/dev/zero of=/dev/sda",
            "chmod 777 /",
            "> /etc/passwd",
            // "mkfs.ext4 /dev/sda1", // This is detected as Caution
        ];

        for cmd in &dangerous_commands {
            assert_eq!(validator.check_safety_level(cmd), SafetyLevel::Dangerous);
        }

        // Test separately the command that is detected as Caution
        assert_eq!(
            validator.check_safety_level("mkfs.ext4 /dev/sda1"),
            SafetyLevel::Caution
        );
    }

    #[test]
    fn test_validate_command() {
        let validator = CommandSafetyValidator::default();

        // Test upgrading safety level
        let mut cmd1 = create_test_command("rm -rf /tmp/*");
        validator.validate(&mut cmd1);
        assert_eq!(cmd1.safety_level, SafetyLevel::Dangerous); // Detected as Dangerous

        let mut cmd2 = create_test_command("rm -rf /");
        validator.validate(&mut cmd2);
        assert_eq!(cmd2.safety_level, SafetyLevel::Dangerous);

        // Test not downgrading safety level
        let mut cmd3 = create_test_command("echo 'hello'");
        cmd3.safety_level = SafetyLevel::Dangerous; // LLM marked as dangerous for some reason
        validator.validate(&mut cmd3);
        assert_eq!(cmd3.safety_level, SafetyLevel::Dangerous); // Should stay dangerous
    }
}
