use std::collections::HashMap;
use std::io::{self, Write};

use crate::config::{Config, ProviderConfig};
use crate::providers;
use crate::ui::Theme;

pub fn run_setup(theme: &Theme) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", theme.header("tryto setup wizard"));
    println!("{}", theme.description("Configure your AI provider\n"));

    // Load provider list
    let providers_list = providers::list();

    // Step 1: Select provider
    let provider_idx = select_provider(theme, providers_list)?;
    let provider_def = &providers_list[provider_idx];
    let provider_id = &provider_def.id;

    // Step 2: API key (skip for ollama)
    let api_key = if provider_def.protocol == providers::ProviderProtocol::Ollama {
        None
    } else {
        Some(read_api_key(theme, provider_id)?)
    };

    // Step 3: Select or enter model
    let models: Vec<&str> = provider_def
        .models
        .iter()
        .map(|s: &String| s.as_str())
        .collect();
    let model = select_model(theme, &models)?;

    // Step 4: Base URL (optional, only for ollama currently)
    let base_url = read_base_url(theme, provider_def)?;

    // Step 5: Create config
    let provider_config = ProviderConfig {
        provider: provider_id.to_string(),
        api_key,
        base_url,
        default_model: model,
    };

    let mut providers_map = HashMap::new();
    providers_map.insert(provider_id.to_string(), provider_config);

    let config = Config {
        default_provider: provider_id.to_string(),
        providers: providers_map,
    };

    // Step 6: Save config
    config.save_default()?;

    println!(
        "\n{} configuration saved to ~/.config/tryto/config.toml",
        theme.success("✓")
    );

    Ok(())
}

fn select_provider(
    theme: &Theme,
    providers_list: &[providers::ProviderDefinition],
) -> Result<usize, Box<dyn std::error::Error>> {
    println!("{}", theme.header("Select a provider:"));
    for (i, def) in providers_list.iter().enumerate() {
        println!(
            "  [{}] {:<12} - {}",
            theme.step_number(i + 1),
            theme.executable(&def.id),
            theme.description(&def.description)
        );
    }

    loop {
        print!("\n{} ", theme.prompt(">"));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(n) if n > 0 && n <= providers_list.len() => return Ok(n - 1),
            _ => println!("{}", theme.error("Please enter a valid number")),
        }
    }
}

fn read_api_key(theme: &Theme, provider: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!(
        "\n{}",
        theme.header(format!("Enter your {} API key:", provider))
    );

    loop {
        print!("{} ", theme.prompt(">"));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let key = input.trim();
        if !key.is_empty() {
            return Ok(key.to_string());
        }
        println!("{}", theme.error("API key cannot be empty"));
    }
}

fn select_model(theme: &Theme, models: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    println!("\n{}", theme.header("Select a model (or type custom):"));

    for (i, model) in models.iter().enumerate() {
        println!(
            "  [{}] {}",
            theme.step_number(i + 1),
            theme.executable(model)
        );
    }
    println!("  [{}] custom", theme.step_number(models.len() + 1));

    loop {
        print!("\n{} ", theme.prompt(">"));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed = input.trim();

        if let Ok(n) = trimmed.parse::<usize>() {
            if n > 0 && n <= models.len() {
                return Ok(models[n - 1].to_string());
            } else if n == models.len() + 1 {
                return read_custom_model(theme);
            }
        }

        // Treat as custom model name
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }

        println!(
            "{}",
            theme.error("Please select a number or enter a model name")
        );
    }
}

fn read_custom_model(theme: &Theme) -> Result<String, Box<dyn std::error::Error>> {
    println!("\n{}", theme.header("Enter model name:"));

    loop {
        print!("{} ", theme.prompt(">"));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let model = input.trim();
        if !model.is_empty() {
            return Ok(model.to_string());
        }
        println!("{}", theme.error("Model name cannot be empty"));
    }
}

fn read_base_url(
    theme: &Theme,
    provider_def: &providers::ProviderDefinition,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Only ask for base URL for Ollama (local) or if there's a default we might
    // want to override
    let default_url = match provider_def.protocol {
        providers::ProviderProtocol::Ollama => provider_def
            .default_base_url
            .as_deref()
            .or(Some("http://localhost:11434")),
        _ => return Ok(None),
    };

    if let Some(url) = default_url {
        println!(
            "\n{} [{}]: ",
            theme.header("Base URL (optional)"),
            theme.hint(url)
        );

        print!("{} ", theme.prompt(">"));
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let user_url = input.trim();
        if user_url.is_empty() {
            Ok(Some(url.to_string()))
        } else {
            Ok(Some(user_url.to_string()))
        }
    } else {
        Ok(None)
    }
}
