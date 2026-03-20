mod config;
mod prompts;
mod response;
mod theme;

use config::{Config, ProviderConfig};
use indicatif::{ProgressBar, ProgressStyle};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use std::env;
use std::time::Duration;
use theme::Theme;

#[tokio::main]
async fn main() {
    let theme = Theme::default();

    // Collect all arguments after the program name
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!(
            "{}: tryto <natural language description>",
            theme.header("usage")
        );
        eprintln!(
            "{}: tryto list files modified in the last 24 hours",
            theme.hint("example")
        );
        std::process::exit(1);
    }

    // Join arguments into a single query string
    let query = args.join(" ");

    // Load configuration
    let config = match Config::load_default() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}: {}", theme.error("error"), e);
            eprintln!(
                "{}: make sure ~/.config/tryto/config.toml exists",
                theme.hint("hint")
            );
            std::process::exit(1);
        }
    };

    // Get the default provider configuration
    let provider_config = match config.get_default_provider() {
        Some(provider) => provider,
        None => {
            eprintln!(
                "{}: default provider '{}' not found in configuration",
                theme.error("error"),
                config.default_provider
            );
            std::process::exit(1);
        }
    };

    // Generate command using AI
    let resp = match generate_command(provider_config, &query).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{}: {}", theme.error("error"), e);
            std::process::exit(1);
        }
    };

    // Write command line to tmp log file for debugging
    let log_path = std::path::PathBuf::from("/tmp/tryto_debug.log");
    if let Err(e) = std::fs::write(&log_path, &resp.command_line) {
        eprintln!(
            "{}: failed to write debug log: {}",
            theme.warning("warning"),
            e
        );
    }

    // Show the pipeline info with descriptions
    resp.pipeline.iter().for_each(|cmd| {
        println!(
            "{} - {}",
            theme.executable(&cmd.executable),
            theme.description(&cmd.description)
        );
        for arg in &cmd.args {
            println!(
                "  {} {}",
                theme.argument(format!("{:<4}", arg.name)),
                theme.description(&arg.description)
            );
        }
        println!();
    });
    println!("$ {}", theme.command_line(&resp.command_line));
    print!("\n{} ", theme.prompt("Execute? [Y/n]"));
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
        println!("{}", theme.hint("command cancelled"));
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
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Generating command...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let provider_type = provider_config.provider.as_str();

    let result = match provider_type {
        "anthropic" => {
            use rig::providers::anthropic;

            let api_key = provider_config
                .api_key
                .as_deref()
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

            agent.prompt(query).await
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

            agent.prompt(query).await
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

            agent.prompt(query).await
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

            agent.prompt(query).await
        }
        "ollama" => {
            use rig::client::Nothing;
            use rig::providers::ollama;

            let client = ollama::Client::new(Nothing)?;

            let agent = client
                .agent(&provider_config.default_model)
                .preamble(prompts::COMMAND_GENERATOR)
                .build();

            agent.prompt(query).await
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

            agent.prompt(query).await
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

            agent.prompt(query).await
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

            agent.prompt(query).await
        }
        _ => {
            spinner.finish_and_clear();
            return Err(format!("Unknown provider: {}", provider_type).into());
        }
    };

    spinner.finish_and_clear();
    let response = result?;
    Ok(parse_response(&response)?)
}
