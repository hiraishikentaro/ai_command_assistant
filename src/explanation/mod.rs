//! Command explanation display functionality (FR-003)
//!
//! This module provides formatted display of command explanations

pub mod components;
pub mod formatter;
pub mod styles;

pub use components::OptionExplanation;
pub use formatter::CommandExplanationFormatter;
pub use styles::ExplanationStyle;
