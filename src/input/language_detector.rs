use unicode_segmentation::UnicodeSegmentation;

/// Target languages for detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    /// Japanese
    Japanese,
    /// English
    English,
    /// Other or unknown
    Other,
}

/// Language detection functionality
pub struct LanguageDetector;

impl LanguageDetector {
    /// Detect the language of the given text
    pub fn detect(text: &str) -> Language {
        // Treat empty input as Other
        if text.trim().is_empty() {
            return Language::Other;
        }

        // Counter for Japanese characters (Kanji, Hiragana, Katakana)
        let mut jp_chars = 0;
        // Counter for English characters
        let mut en_chars = 0;
        // Counter for other characters
        let mut _other_chars = 0;

        // Use Unicode grapheme clusters for accurate character segmentation
        for grapheme in text.graphemes(true) {
            if Self::is_japanese_char(grapheme) {
                jp_chars += 1;
            } else if Self::is_english_char(grapheme) {
                en_chars += 1;
            } else {
                _other_chars += 1;
            }
        }

        // Return the language with the most characters
        // If there are Japanese characters above a threshold, return Japanese
        if jp_chars > 0 && jp_chars >= en_chars {
            Language::Japanese
        } else if en_chars > 0 && en_chars > jp_chars {
            Language::English
        } else {
            Language::Other
        }
    }

    /// Check if a character is Japanese
    fn is_japanese_char(grapheme: &str) -> bool {
        // Use Unicode blocks to detect Japanese characters
        grapheme.chars().any(|c| {
            // Hiragana
            (c >= '\u{3040}' && c <= '\u{309F}') ||
            // Katakana
            (c >= '\u{30A0}' && c <= '\u{30FF}') ||
            // CJK Unified Ideographs (Kanji)
            (c >= '\u{4E00}' && c <= '\u{9FFF}')
        })
    }

    /// Check if a character is English
    fn is_english_char(grapheme: &str) -> bool {
        // English letters (uppercase and lowercase)
        grapheme
            .chars()
            .any(|c| (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_japanese() {
        assert_eq!(
            LanguageDetector::detect("こんにちは世界"),
            Language::Japanese
        );
        assert_eq!(
            LanguageDetector::detect("ファイルを検索"),
            Language::Japanese
        );
        assert_eq!(
            LanguageDetector::detect("テキスト処理について"),
            Language::Japanese
        );
    }

    #[test]
    fn test_detect_english() {
        assert_eq!(LanguageDetector::detect("Hello World"), Language::English);
        assert_eq!(LanguageDetector::detect("Find files"), Language::English);
        assert_eq!(
            LanguageDetector::detect("About text processing"),
            Language::English
        );
    }

    #[test]
    fn test_detect_mixed() {
        // When Japanese characters are more prevalent
        assert_eq!(
            LanguageDetector::detect("Rustでファイル処理をする"),
            Language::Japanese
        );
        // When English characters are more prevalent
        assert_eq!(
            LanguageDetector::detect("Run ls コマンド"),
            Language::English
        );
    }

    #[test]
    fn test_detect_empty() {
        assert_eq!(LanguageDetector::detect(""), Language::Other);
        assert_eq!(LanguageDetector::detect("   "), Language::Other);
    }
}
