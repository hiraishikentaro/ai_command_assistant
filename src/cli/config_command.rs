use crate::llm::LlmConfig;
use crate::llm::providers::LlmProvider;
use anyhow::{Result, anyhow};
use colored::Colorize;
use promptuity::{
    Promptuity, Term,
    prompts::{Confirm, Input, Select, SelectOption},
    themes::FancyTheme,
};
use std::env;

/// Config command to interactively set up LLM configuration
pub struct ConfigCommand;

impl ConfigCommand {
    /// Run the config command
    pub async fn run() -> Result<()> {
        println!(
            "{}\n",
            "AI Command Assistant Configuration".bright_cyan().bold()
        );
        println!(
            "{}",
            "This will help you set up the LLM API providers.".bright_cyan()
        );
        println!(
            "{}\n",
            "You can configure multiple providers and switch between them.".bright_cyan()
        );

        // Initialize promptuity with a fancy theme
        let mut term = Term::stderr();
        let mut theme = FancyTheme::default();
        let mut promptuity = Promptuity::new(&mut term, &mut theme);

        // Setup promptuity
        promptuity.begin()?;

        // Load existing config if available
        let mut config = match LlmConfig::load() {
            Ok(config) => config,
            Err(_) => LlmConfig::default(),
        };

        // Choose provider to configure
        let provider_choices: Vec<SelectOption<LlmProvider>> = [
            LlmProvider::OpenAI,
            LlmProvider::Gemini,
            LlmProvider::Claude,
        ]
        .iter()
        .map(|p| SelectOption::new(p.display_name(), p.clone()))
        .collect();

        let mut select_provider = Select::new(
            "Which LLM provider would you like to configure?",
            provider_choices,
        );
        let provider = promptuity.prompt(select_provider.as_mut())?;

        println!();
        println!(
            "{}\n",
            format!("Configuring {}...", provider.display_name()).bright_cyan()
        );

        // Get API key
        let env_var = provider.api_key_env_var();
        let api_key_env = env::var(env_var).unwrap_or_default();

        // Check if it's already in config
        let provider_str = format!("{:?}", provider).to_lowercase();
        let current_key = if let Some(provider_config) = config.providers.get(&provider_str) {
            provider_config.api_key.clone()
        } else {
            String::new()
        };

        let mut api_key_prompt = format!("Enter your {} API key", provider.display_name());
        if !api_key_env.is_empty() {
            api_key_prompt.push_str(" (found in environment)");
        } else if !current_key.is_empty() {
            api_key_prompt.push_str(" (found in config)");
        }

        let default_key = if !api_key_env.is_empty() {
            api_key_env.clone()
        } else if !current_key.is_empty() {
            current_key.clone()
        } else {
            String::new()
        };

        let mut input = Input::new(&api_key_prompt);
        let api_key = promptuity.prompt(input.as_mut())?;

        // 空の場合はデフォルト値を使用
        let api_key = if api_key.is_empty() && !default_key.is_empty() {
            default_key
        } else {
            api_key
        };

        // Get model to use
        let available_models = provider.default_models();
        let model_choices: Vec<SelectOption<String>> = available_models
            .iter()
            .map(|m| SelectOption::new(*m, m.to_string()))
            .collect();

        // Get current model if it exists
        let _current_model = if let Some(provider_config) = config.providers.get(&provider_str) {
            provider_config.model.clone()
        } else {
            available_models[0].to_string()
        };

        let mut select_model = Select::new("Which model would you like to use?", model_choices);
        let selected_model = promptuity.prompt(select_model.as_mut())?;

        // Set as active provider?
        let mut confirm = Confirm::new("Do you want to set this as your active provider?");
        let make_active = promptuity.prompt(confirm.as_mut())?;

        // Update config
        config.set_provider(provider.clone(), api_key, selected_model);

        // Set the active flag
        let provider_str = format!("{:?}", provider).to_lowercase();
        if let Some(provider_config) = config.providers.get_mut(&provider_str) {
            provider_config.is_active = make_active;

            // If not active, we need to find another provider to make active
            if !make_active {
                // Find the current active provider or set a default
                let mut found_active = false;
                for (name, provider_config) in &mut config.providers {
                    if provider_config.is_active && name != &provider_str {
                        found_active = true;

                        // Update main config to match this provider
                        if let Some(provider) = LlmProvider::from_str(name) {
                            config.provider = provider;
                            config.api_key = provider_config.api_key.clone();
                            config.model = provider_config.model.clone();
                        }
                        break;
                    }
                }

                // If no active provider, set the first one as active
                if !found_active && !config.providers.is_empty() {
                    let first_provider = config.providers.keys().next().unwrap().clone();
                    if let Some(provider_config) = config.providers.get_mut(&first_provider) {
                        provider_config.is_active = true;

                        // Update main config to match this provider
                        if let Some(provider) = LlmProvider::from_str(&first_provider) {
                            config.provider = provider;
                            config.api_key = provider_config.api_key.clone();
                            config.model = provider_config.model.clone();
                        }
                    }
                }
            }
        }

        // Finish promptuity
        promptuity.finish()?;

        // Save config
        if let Err(e) = config.save() {
            return Err(anyhow!("Failed to save configuration: {}", e));
        }

        // Success message
        println!("\n{}", "Configuration saved successfully!".bright_green());
        println!(
            "\n{} {}\n",
            "Active provider:".bright_cyan(),
            config.provider.display_name().bright_yellow()
        );

        // Set environment variable recommendation
        println!(
            "{}",
            "For better security, you can also set your API key as an environment variable:"
                .bright_cyan()
        );
        println!(
            "export {}='your-api-key-here'\n",
            provider.api_key_env_var().bright_yellow()
        );

        Ok(())
    }
}
