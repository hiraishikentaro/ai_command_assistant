use std::env;
use std::fs;
use std::path::PathBuf;

use crate::llm::error::LlmError;
use dirs;
use log::debug;

/// LLMのAPIリクエスト用設定
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub timeout_seconds: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: 500,
            temperature: 0.2,
            top_p: 1.0,
            timeout_seconds: 30,
        }
    }
}

impl LlmConfig {
    /// 新しい設定インスタンスを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 環境変数からAPIキーを読み込む
    pub fn load_api_key(&mut self) -> Result<(), LlmError> {
        // 環境変数からAPIキーを取得
        if let Ok(api_key) = env::var("ACIA_OPENAI_API_KEY") {
            if !api_key.is_empty() {
                self.api_key = api_key;
                debug!("Loaded API key from environment variable");
                return Ok(());
            }
        }

        // 設定ファイルからAPIキーを取得
        if let Some(config_path) = self.get_config_file_path() {
            if config_path.exists() {
                match fs::read_to_string(&config_path) {
                    Ok(content) => {
                        if let Some(api_key) = self.extract_api_key_from_config(&content) {
                            self.api_key = api_key;
                            debug!("Loaded API key from config file: {:?}", config_path);
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        debug!("Failed to read config file: {}", e);
                    }
                }
            }
        }

        // APIキーが取得できなかった場合はエラー
        if self.api_key.is_empty() {
            return Err(LlmError::ApiKeyNotSet);
        }

        Ok(())
    }

    /// 設定ファイルのパスを取得
    fn get_config_file_path(&self) -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| {
            let mut path = config_dir;
            path.push("acia");
            path.push("config.toml");
            path
        })
    }

    /// 設定ファイルからAPIキーを抽出
    fn extract_api_key_from_config(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("openai_api_key") || line.starts_with("api_key") {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() >= 2 {
                    let value = parts[1].trim();
                    // 引用符を取り除く
                    let value = value.trim_matches('"').trim_matches('\'');
                    if !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }

    /// 使用するモデルを設定
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// 温度パラメータを設定
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// 最大トークン数を設定
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// タイムアウト秒数を設定
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.max_tokens, 500);
        assert_eq!(config.temperature, 0.2);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_builder_pattern() {
        let config = LlmConfig::new()
            .with_model("gpt-4")
            .with_temperature(0.7)
            .with_max_tokens(1000)
            .with_timeout(60);

        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_tokens, 1000);
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_extract_api_key_from_config() {
        let config = LlmConfig::default();

        // 通常のケース
        let content = r#"
        [openai]
        openai_api_key = "sk-1234567890abcdef"
        "#;
        assert_eq!(
            config.extract_api_key_from_config(content),
            Some("sk-1234567890abcdef".to_string())
        );

        // シングルクォートのケース
        let content = r#"
        api_key = 'sk-abcdef1234567890'
        "#;
        assert_eq!(
            config.extract_api_key_from_config(content),
            Some("sk-abcdef1234567890".to_string())
        );

        // 空の場合
        let content = r#"
        [openai]
        openai_api_key = ""
        "#;
        assert_eq!(config.extract_api_key_from_config(content), None);
    }
}
