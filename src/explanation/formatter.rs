use crate::explanation::components::{build_explanation_from_candidate, extract_related_commands};
use crate::explanation::styles::ExplanationStyle;
use crate::llm::CommandCandidate;
use textwrap::fill;
use unicode_width::UnicodeWidthStr;

/// Command explanation formatter
pub struct CommandExplanationFormatter {
    /// Style to use for formatting
    style: ExplanationStyle,
    /// Terminal width for text wrapping
    terminal_width: usize,
    /// Whether to show related commands
    show_related: bool,
}

impl Default for CommandExplanationFormatter {
    fn default() -> Self {
        Self {
            style: ExplanationStyle::default(),
            terminal_width: 80, // Default terminal width
            show_related: true,
        }
    }
}

impl CommandExplanationFormatter {
    /// Create a new formatter with custom style
    pub fn new(style: ExplanationStyle) -> Self {
        Self {
            style,
            ..Default::default()
        }
    }

    /// Set the terminal width for text wrapping
    pub fn with_terminal_width(mut self, width: usize) -> Self {
        self.terminal_width = width;
        self
    }

    /// Set whether to show related commands
    pub fn with_related_commands(mut self, show: bool) -> Self {
        self.show_related = show;
        self
    }

    /// Format a command explanation for display
    pub fn format(&self, candidate: &CommandCandidate) -> String {
        let mut output = String::new();

        // Format COMMAND section
        output.push_str(&format!(
            "{} {}\n\n",
            (self.style.heading_style)("COMMAND:"),
            (self.style.command_style)(&candidate.command, &candidate.safety_level)
        ));

        // Format PURPOSE section
        output.push_str(&format!(
            "{} {}\n\n",
            (self.style.heading_style)("PURPOSE:"),
            fill(&candidate.explanation.purpose, self.terminal_width - 10)
        ));

        // Format OPTIONS section
        let options = build_explanation_from_candidate(candidate);
        if !options.is_empty() {
            output.push_str(&format!("{}\n", (self.style.heading_style)("OPTIONS:")));

            // Calculate the maximum width of the option text for alignment
            let max_option_width = options
                .iter()
                .map(|opt| UnicodeWidthStr::width(opt.option_text.as_str()))
                .max()
                .unwrap_or(0);

            for opt in options {
                // Format each option with proper indentation
                let opt_text = (self.style.option_style)(&opt.option_text);

                // Wrap the explanation text
                let wrapped_explanation = fill(
                    &opt.explanation,
                    self.terminal_width - max_option_width - 10,
                );

                let padding = " ".repeat(4); // 4 spaces indentation

                // First line with option and start of explanation
                output.push_str(&format!(
                    "{}{}: {}\n",
                    padding, opt_text, wrapped_explanation
                ));
            }

            output.push('\n');
        }

        // Format RELATED section if enabled
        if self.show_related {
            let related = extract_related_commands(&candidate.command);
            if !related.is_empty() {
                output.push_str(&format!("{} ", (self.style.heading_style)("RELATED:")));

                let related_cmds: Vec<_> = related
                    .iter()
                    .map(|cmd| (self.style.related_style)(cmd).to_string())
                    .collect();

                output.push_str(&related_cmds.join(", "));
                output.push('\n');
            }
        }

        output
    }

    /// Print the formatted explanation to stdout
    pub fn print(&self, candidate: &CommandCandidate) {
        print!("{}", self.format(candidate));
    }
}

/// Format a command explanation with the default style
pub fn format_command_explanation(candidate: &CommandCandidate) -> String {
    CommandExplanationFormatter::default().format(candidate)
}

/// Print a command explanation with the default style
pub fn print_command_explanation(candidate: &CommandCandidate) {
    CommandExplanationFormatter::default().print(candidate);
}
