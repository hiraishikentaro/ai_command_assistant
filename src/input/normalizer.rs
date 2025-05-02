use regex::Regex;
use std::borrow::Cow;

/// Text normalization functionality
pub struct TextNormalizer;

impl TextNormalizer {
    /// Normalize the input text
    ///
    /// - Remove leading and trailing whitespace
    /// - Replace multiple consecutive whitespace with a single space
    /// - Normalize punctuation marks
    pub fn normalize(text: &str) -> String {
        let text = text.trim();

        // Return empty string for empty input
        if text.is_empty() {
            return String::new();
        }

        // Replace multiple consecutive whitespace with a single space
        let normalized = Self::normalize_whitespace(text);

        // Normalize punctuation (convert Japanese full-width punctuation to half-width, etc.)
        let normalized = Self::normalize_punctuation(&normalized);

        normalized.to_string()
    }

    /// Normalize whitespace (replace multiple consecutive whitespace with a single space)
    fn normalize_whitespace(text: &str) -> Cow<'_, str> {
        lazy_static::lazy_static! {
            static ref WHITESPACE_REGEX: Regex = Regex::new(r"\s+").unwrap();
        }

        WHITESPACE_REGEX.replace_all(text, " ")
    }

    /// Normalize punctuation
    fn normalize_punctuation(text: &str) -> Cow<'_, str> {
        // Convert full-width Japanese punctuation to half-width
        let text = text.replace('、', ",").replace('。', ".");

        // Convert full-width brackets to half-width
        let text = text.replace('（', "(").replace('）', ")");

        // Convert other full-width symbols to half-width
        let text = text
            .replace('：', ":")
            .replace('；', ";")
            .replace('！', "!")
            .replace('？', "?");

        Cow::Owned(text)
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(
            super::TextNormalizer::normalize_whitespace("hello   world"),
            "hello world"
        );
        assert_eq!(
            super::TextNormalizer::normalize_whitespace("line1\n  line2\t\tline3"),
            "line1 line2 line3"
        );
    }

    #[test]
    fn test_normalize_punctuation() {
        assert_eq!(
            super::TextNormalizer::normalize_punctuation("こんにちは、世界。"),
            "こんにちは,世界."
        );
        assert_eq!(
            super::TextNormalizer::normalize_punctuation("テスト（コメント）"),
            "テスト(コメント)"
        );
    }

    #[test]
    fn test_normalize() {
        assert_eq!(
            TextNormalizer::normalize("  hello   world  "),
            "hello world"
        );
        assert_eq!(
            TextNormalizer::normalize("こんにちは、世界。  新しい行"),
            "こんにちは,世界. 新しい行"
        );
        assert_eq!(TextNormalizer::normalize(""), "");
    }
}
