mod config;

use config::Config;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::anthropic;

#[tokio::main]
async fn main() {
    // Try to load configuration from default location
    let config = match Config::load_default() {
        Ok(cfg) => {
            println!("Loaded configuration successfully");
            println!("Default provider: {}", cfg.default_provider);
            cfg
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Make sure ~/.config/tryto/config.toml exists");
            std::process::exit(1);
        }
    };

    // Get the default provider configuration
    let provider_config = match config.get_default_provider() {
        Some(provider) => {
            println!("Provider type: {}", provider.provider);
            println!("Base URL: {:?}", provider.base_url);
            println!("Default model: {}", provider.default_model);
            provider
        }
        None => {
            eprintln!(
                "Default provider '{}' not found in configuration",
                config.default_provider
            );
            std::process::exit(1);
        }
    };

    // Test the provider with a simple API call
    println!("\nTesting API connection...");

    // Use Anthropic-compatible client for kimi coding API
    let client = anthropic::Client::builder()
        .api_key(provider_config.api_key.as_deref().unwrap_or(""))
        .base_url(provider_config.base_url.as_deref().unwrap_or("https://api.anthropic.com"))
        .build()
        .expect("Failed to build Anthropic client");

    let agent = client
        .agent(&provider_config.default_model)
        .preamble("You are a helpful assistant.")
        .max_tokens(1024)
        .build();

    match agent.prompt("Say hello and tell me who you are in one sentence.").await {
        Ok(response) => {
            println!("\n✅ API call succeeded!");
            println!("Response: {}", response);
        }
        Err(e) => {
            eprintln!("\n❌ API call failed: {}", e);
            std::process::exit(1);
        }
    }
}
