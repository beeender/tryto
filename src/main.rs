mod config;
mod prompts;
mod response;

use config::{Config, ProviderConfig};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use std::env;

#[tokio::main]
async fn main() {
    // Collect all arguments after the program name
    let args: Vec<String> = env::args().skip(1).collect();
    
    if args.is_empty() {
        eprintln!("Usage: tryto <natural language description of what you want to do>");
        eprintln!("Example: tryto list all files modified in the last 24 hours");
        std::process::exit(1);
    }

    // Join arguments into a single query string
    let query = args.join(" ");

    // Load configuration
    let config = match Config::load_default() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Make sure ~/.config/tryto/config.toml exists");
            std::process::exit(1);
        }
    };

    // Get the default provider configuration
    let provider_config = match config.get_default_provider() {
        Some(provider) => provider,
        None => {
            eprintln!(
                "Default provider '{}' not found in configuration",
                config.default_provider
            );
            std::process::exit(1);
        }
    };

    // Generate command using AI
    let resp = match generate_command(provider_config, &query).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to generate command: {}", e);
            std::process::exit(1);
        }
    };

    // Write command line to tmp log file for debugging
    let log_path = std::path::PathBuf::from("/tmp/tryto_debug.log");
    if let Err(e) = std::fs::write(&log_path, &resp.command_line) {
        eprintln!("Warning: Failed to write debug log: {}", e);
    } else {
        eprintln!("Debug: Command written to {}", log_path.display());
    }

    // Show the pipeline info with descriptions
    println!("\nCommand pipeline:");
    for (i, cmd) in resp.pipeline.iter().enumerate() {
        println!("  [{}] {} - {}", i + 1, cmd.executable, cmd.description);
        for arg in &cmd.args {
            println!("      {:<15} {}", arg.name, arg.description);
        }
    }
    println!("\n$ {}", resp.command_line);
    print!("\nExecute? [Y/n] ");
    use std::io::Write;
    std::io::stdout().flush().unwrap();
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    
    let input = input.trim().to_lowercase();
    if input.is_empty() || input == "y" || input == "yes" {
        // Execute the command
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&resp.command_line)
            .status()
            .expect("Failed to execute command");

        std::process::exit(status.code().unwrap_or(1));
    } else {
        println!("Command cancelled");
    }
}

fn parse_response(response: &str) -> Result<response::Response, Box<dyn std::error::Error>> {
    // Try to parse as JSON first
    let json_str = response.trim();
    // Handle markdown code blocks
    let json_str = if json_str.starts_with("```") {
        json_str
            .lines()
            .skip(1)
            .take_while(|line| !line.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        json_str.to_string()
    };

    let resp: response::Response = serde_json::from_str(&json_str)?;
    Ok(resp)
}

async fn generate_command(
    provider_config: &ProviderConfig,
    query: &str,
) -> Result<response::Response, Box<dyn std::error::Error>> {
    let provider_type = provider_config.provider.as_str();
    
    let response = match provider_type {
        "anthropic" => {
            use rig::providers::anthropic;
            
            let api_key = provider_config.api_key.as_deref()
                .ok_or("API key is required for Anthropic provider")?;
            
            let client = if let Some(ref base_url) = provider_config.base_url {
                anthropic::Client::builder()
                    .api_key(api_key)
                    .base_url(base_url)
                    .build()?
            } else {
                anthropic::Client::new(api_key)?
            };

            let agent = client
                .agent(&provider_config.default_model)
                .preamble(prompts::COMMAND_GENERATOR)
                .max_tokens(1024)
                .build();

            agent.prompt(query).await?
        }
        "openai" => {
            use rig::providers::openai;
            
            let client = if let Some(ref api_key) = provider_config.api_key {
                openai::Client::new(api_key)?
            } else {
                openai::Client::from_env()
            };

            let agent = client
                .agent(&provider_config.default_model)
                .preamble(prompts::COMMAND_GENERATOR)
                .max_tokens(1024)
                .build();

            agent.prompt(query).await?
        }
        "deepseek" => {
            use rig::providers::deepseek;
            
            let client = if let Some(ref api_key) = provider_config.api_key {
                deepseek::Client::new(api_key)?
            } else {
                deepseek::Client::from_env()
            };

            let agent = client
                .agent(&provider_config.default_model)
                .preamble(prompts::COMMAND_GENERATOR)
                .max_tokens(1024)
                .build();

            agent.prompt(query).await?
        }
        "gemini" => {
            use rig::providers::gemini;
            
            let client = if let Some(ref api_key) = provider_config.api_key {
                gemini::Client::new(api_key)?
            } else {
                gemini::Client::from_env()
            };

            let agent = client
                .agent(&provider_config.default_model)
                .preamble(prompts::COMMAND_GENERATOR)
                .max_tokens(1024)
                .build();

            agent.prompt(query).await?
        }
        "ollama" => {
            use rig::providers::ollama;
            use rig::client::Nothing;
            
            let client = ollama::Client::new(Nothing)?;

            let agent = client
                .agent(&provider_config.default_model)
                .preamble(prompts::COMMAND_GENERATOR)
                .build();

            agent.prompt(query).await?
        }
        "xai" => {
            use rig::providers::xai;
            
            let client = if let Some(ref api_key) = provider_config.api_key {
                xai::Client::new(api_key)?
            } else {
                xai::Client::from_env()
            };

            let agent = client
                .agent(&provider_config.default_model)
                .preamble(prompts::COMMAND_GENERATOR)
                .max_tokens(1024)
                .build();

            agent.prompt(query).await?
        }
        "perplexity" => {
            use rig::providers::perplexity;
            
            let client = if let Some(ref api_key) = provider_config.api_key {
                perplexity::Client::new(api_key)?
            } else {
                perplexity::Client::from_env()
            };

            let agent = client
                .agent(&provider_config.default_model)
                .preamble(prompts::COMMAND_GENERATOR)
                .max_tokens(1024)
                .build();

            agent.prompt(query).await?
        }
        "groq" => {
            use rig::providers::groq;
            
            let client = if let Some(ref api_key) = provider_config.api_key {
                groq::Client::new(api_key)?
            } else {
                groq::Client::from_env()
            };

            let agent = client
                .agent(&provider_config.default_model)
                .preamble(prompts::COMMAND_GENERATOR)
                .max_tokens(1024)
                .build();

            agent.prompt(query).await?
        }
        _ => {
            return Err(format!("Unknown provider: {}", provider_type).into());
        }
    };
    
    Ok(parse_response(&response)?)
}
