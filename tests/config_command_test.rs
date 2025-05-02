mod common;

use acia::llm::LlmConfig;
use acia::llm::providers::LlmProvider;

#[test]
fn test_llmconfig_provider_setup() {
    // 新しい設定を作成
    let mut config = LlmConfig::default();

    // OpenAI プロバイダーを設定
    config.set_provider(
        LlmProvider::OpenAI,
        "test-openai-api-key".to_string(),
        "gpt-4".to_string(),
    );

    // 設定がプロバイダーを正しく設定したことを確認
    assert_eq!(config.provider, LlmProvider::OpenAI);
    assert_eq!(config.api_key, "test-openai-api-key");
    assert_eq!(config.model, "gpt-4");

    // プロバイダー設定を確認
    let provider_str = format!("{:?}", LlmProvider::OpenAI).to_lowercase();
    let provider_config = config.providers.get(&provider_str).unwrap();
    assert_eq!(provider_config.api_key, "test-openai-api-key");
    assert_eq!(provider_config.model, "gpt-4");
    assert!(provider_config.is_active);
}

#[test]
fn test_multiple_providers() {
    // 複数のプロバイダー設定を持つ設定をテスト
    let mut config = LlmConfig::default();

    // OpenAI プロバイダーを設定
    config.set_provider(
        LlmProvider::OpenAI,
        "test-openai-api-key".to_string(),
        "gpt-4".to_string(),
    );

    // Gemini プロバイダーを設定（アクティブとしてマーク）
    config.set_provider(
        LlmProvider::Gemini,
        "test-gemini-api-key".to_string(),
        "gemini-pro".to_string(),
    );

    // Claude プロバイダーを設定（非アクティブ）
    let claude_api_key = "test-claude-api-key".to_string();
    let claude_model = "claude-3-opus".to_string();
    config.set_provider(
        LlmProvider::Claude,
        claude_api_key.clone(),
        claude_model.clone(),
    );

    // Claude プロバイダーを非アクティブにする
    let provider_str = format!("{:?}", LlmProvider::Claude).to_lowercase();
    if let Some(provider_config) = config.providers.get_mut(&provider_str) {
        provider_config.is_active = false;
    }

    // Gemini が現在のアクティブプロバイダーではなく、Claude が最後に設定されたプロバイダーであることを確認
    // (LlmConfig の実装では、最後に set_provider を呼んだプロバイダーがアクティブになります)
    assert_eq!(config.provider, LlmProvider::Claude);
    assert_eq!(config.api_key, claude_api_key);
    assert_eq!(config.model, claude_model);

    // すべてのプロバイダーが設定に存在することを確認
    assert_eq!(config.providers.len(), 3);

    // OpenAI プロバイダーの確認（非アクティブ）
    let openai_str = format!("{:?}", LlmProvider::OpenAI).to_lowercase();
    let openai_config = config.providers.get(&openai_str).unwrap();
    assert_eq!(openai_config.api_key, "test-openai-api-key");
    assert_eq!(openai_config.model, "gpt-4");
    assert!(!openai_config.is_active);

    // Gemini プロバイダーの確認（非アクティブ）
    let gemini_str = format!("{:?}", LlmProvider::Gemini).to_lowercase();
    let gemini_config = config.providers.get(&gemini_str).unwrap();
    assert_eq!(gemini_config.api_key, "test-gemini-api-key");
    assert_eq!(gemini_config.model, "gemini-pro");
    assert!(!gemini_config.is_active);

    // Claude プロバイダーの確認（非アクティブに設定した）
    let claude_str = format!("{:?}", LlmProvider::Claude).to_lowercase();
    let claude_config = config.providers.get(&claude_str).unwrap();
    assert_eq!(claude_config.api_key, claude_api_key);
    assert_eq!(claude_config.model, claude_model);
    assert!(!claude_config.is_active);
}

#[test]
fn test_provider_display_names() {
    // プロバイダーの表示名をテスト
    assert_eq!(LlmProvider::OpenAI.display_name(), "OpenAI");
    assert_eq!(LlmProvider::Gemini.display_name(), "Google Gemini");
    assert_eq!(LlmProvider::Claude.display_name(), "Anthropic Claude");
}

#[test]
fn test_provider_env_vars() {
    // プロバイダーの環境変数名をテスト
    assert_eq!(LlmProvider::OpenAI.api_key_env_var(), "ACIA_OPENAI_API_KEY");
    assert_eq!(LlmProvider::Gemini.api_key_env_var(), "ACIA_GEMINI_API_KEY");
    assert_eq!(LlmProvider::Claude.api_key_env_var(), "ACIA_CLAUDE_API_KEY");
}

#[test]
fn test_provider_from_str() {
    // 文字列からプロバイダーへの変換をテスト
    assert_eq!(LlmProvider::from_str("openai"), Some(LlmProvider::OpenAI));
    assert_eq!(LlmProvider::from_str("OpenAI"), Some(LlmProvider::OpenAI));
    assert_eq!(LlmProvider::from_str("OPENAI"), Some(LlmProvider::OpenAI));

    assert_eq!(LlmProvider::from_str("gemini"), Some(LlmProvider::Gemini));
    assert_eq!(LlmProvider::from_str("Gemini"), Some(LlmProvider::Gemini));

    assert_eq!(LlmProvider::from_str("claude"), Some(LlmProvider::Claude));
    assert_eq!(LlmProvider::from_str("Claude"), Some(LlmProvider::Claude));

    assert_eq!(LlmProvider::from_str("unknown"), None);
    assert_eq!(LlmProvider::from_str(""), None);
}

#[test]
fn test_default_models() {
    // プロバイダーのデフォルトモデルをテスト
    let openai_models = LlmProvider::OpenAI.default_models();
    assert!(openai_models.contains(&"gpt-3.5-turbo"));
    assert!(openai_models.contains(&"gpt-4"));

    let gemini_models = LlmProvider::Gemini.default_models();
    assert!(gemini_models.contains(&"gemini-pro"));

    let claude_models = LlmProvider::Claude.default_models();
    assert!(claude_models.contains(&"claude-3-opus"));
    assert!(claude_models.contains(&"claude-3-sonnet"));
}

#[test]
fn test_config_builder_pattern() {
    // ビルダーパターンの動作をテスト
    let config = LlmConfig::new()
        .with_model("custom-model")
        .with_temperature(0.8)
        .with_max_tokens(2000)
        .with_timeout(120);

    assert_eq!(config.model, "custom-model");
    assert_eq!(config.temperature, 0.8);
    assert_eq!(config.max_tokens, 2000);
    assert_eq!(config.timeout_seconds, 120);
}

#[test]
fn test_config_default_values() {
    // デフォルト値をテスト
    let config = LlmConfig::default();

    assert_eq!(config.model, "gpt-3.5-turbo");
    assert_eq!(config.max_tokens, 500);
    assert_eq!(config.temperature, 0.2);
    assert_eq!(config.top_p, 1.0);
    assert_eq!(config.timeout_seconds, 30);
    assert_eq!(config.provider, LlmProvider::OpenAI);
    assert!(config.providers.is_empty());
}
