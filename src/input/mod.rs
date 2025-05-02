pub mod language_detector;
pub mod normalizer;
pub mod processor;

// Re-export types for convenient access
pub use language_detector::Language;
pub use processor::InputMode;
pub use processor::InputProcessor;
pub use processor::UserInput;
