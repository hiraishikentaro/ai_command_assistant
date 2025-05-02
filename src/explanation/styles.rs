use crate::llm::SafetyLevel;
use colored::{ColoredString, Colorize};
use std::sync::Arc;

/// Style configuration for command explanations
pub struct ExplanationStyle {
    /// Style for headings like COMMAND, PURPOSE, etc.
    pub heading_style: Arc<dyn Fn(&str) -> ColoredString + Send + Sync>,
    /// Style for the command text
    pub command_style: Arc<dyn Fn(&str, &SafetyLevel) -> ColoredString + Send + Sync>,
    /// Style for option text
    pub option_style: Arc<dyn Fn(&str) -> ColoredString + Send + Sync>,
    /// Style for related commands
    pub related_style: Arc<dyn Fn(&str) -> ColoredString + Send + Sync>,
}

impl Clone for ExplanationStyle {
    fn clone(&self) -> Self {
        Self {
            heading_style: Arc::clone(&self.heading_style),
            command_style: Arc::clone(&self.command_style),
            option_style: Arc::clone(&self.option_style),
            related_style: Arc::clone(&self.related_style),
        }
    }
}

impl Default for ExplanationStyle {
    fn default() -> Self {
        Self {
            heading_style: Arc::new(|text| text.bright_blue()),
            command_style: Arc::new(|text, safety| match safety {
                SafetyLevel::Safe => text.bright_green(),
                SafetyLevel::Caution => text.bright_yellow(),
                SafetyLevel::Dangerous => text.bright_red(),
            }),
            option_style: Arc::new(|text| text.bright_cyan()),
            related_style: Arc::new(|text| text.bright_cyan()),
        }
    }
}

/// Basic style with less colors for terminals with limited color support
pub fn basic_style() -> ExplanationStyle {
    ExplanationStyle {
        heading_style: Arc::new(|text| text.bold()),
        command_style: Arc::new(|text, safety| match safety {
            SafetyLevel::Safe => text.normal(),
            SafetyLevel::Caution => text.bold(),
            SafetyLevel::Dangerous => text.bold(),
        }),
        option_style: Arc::new(|text| text.normal()),
        related_style: Arc::new(|text| text.normal()),
    }
}
