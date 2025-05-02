use crate::llm::response::CommandComponent;
use serde::{Deserialize, Serialize};

/// Option explanation for a specific part of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionExplanation {
    /// Option part text
    pub option_text: String,
    /// Explanation of the option
    pub explanation: String,
}

/// Converts from the LLM API response format to our display format
impl From<CommandComponent> for OptionExplanation {
    fn from(component: CommandComponent) -> Self {
        Self {
            option_text: component.part,
            explanation: component.explanation,
        }
    }
}

/// Builds a formatted explanation from a command candidate
pub fn build_explanation_from_candidate(
    cmd: &crate::llm::CommandCandidate,
) -> Vec<OptionExplanation> {
    cmd.explanation
        .components
        .iter()
        .map(|comp| OptionExplanation {
            option_text: comp.part.clone(),
            explanation: comp.explanation.clone(),
        })
        .collect()
}

/// Extracts related commands based on the command content
pub fn extract_related_commands(cmd: &str) -> Vec<String> {
    // This is a simple implementation that could be expanded in the future
    // to better detect related commands based on the command content

    let mut related = Vec::new();

    // Extract main command (before any space)
    if let Some(main_cmd) = cmd.split_whitespace().next() {
        match main_cmd {
            "ls" => {
                related.extend(vec![
                    "find".to_string(),
                    "dir".to_string(),
                    "tree".to_string(),
                ]);
            }
            "grep" => {
                related.extend(vec![
                    "find".to_string(),
                    "sed".to_string(),
                    "awk".to_string(),
                ]);
            }
            "find" => {
                related.extend(vec![
                    "ls".to_string(),
                    "grep".to_string(),
                    "locate".to_string(),
                ]);
            }
            "cp" | "mv" | "rm" => {
                related.extend(vec![
                    "rsync".to_string(),
                    "mkdir".to_string(),
                    "touch".to_string(),
                ]);
            }
            "cat" => {
                related.extend(vec![
                    "less".to_string(),
                    "more".to_string(),
                    "head".to_string(),
                    "tail".to_string(),
                ]);
            }
            "ps" => {
                related.extend(vec![
                    "top".to_string(),
                    "htop".to_string(),
                    "kill".to_string(),
                ]);
            }
            "wget" | "curl" => {
                related.extend(vec![
                    "curl".to_string(),
                    "wget".to_string(),
                    "http".to_string(),
                ]);
            }
            "tar" => {
                related.extend(vec![
                    "gzip".to_string(),
                    "zip".to_string(),
                    "unzip".to_string(),
                ]);
            }
            // Add more command mappings as needed
            _ => {}
        }
    }

    // If there's a pipe, also consider related commands to the commands after the pipe
    if cmd.contains('|') {
        for part in cmd.split('|').skip(1) {
            if let Some(piped_cmd) = part.trim().split_whitespace().next() {
                match piped_cmd {
                    "grep" => {
                        if !related.contains(&"sed".to_string()) {
                            related.push("sed".to_string());
                        }
                        if !related.contains(&"awk".to_string()) {
                            related.push("awk".to_string());
                        }
                    }
                    "wc" => {
                        related.push("sort".to_string());
                        related.push("uniq".to_string());
                    }
                    "sort" => {
                        related.push("uniq".to_string());
                        related.push("head".to_string());
                    }
                    // Add more piped command mappings
                    _ => {}
                }
            }
        }
    }

    // Remove duplicates
    related.sort();
    related.dedup();

    related
}
