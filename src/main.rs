mod config;
mod prompts;
mod providers;
mod response;
mod setup;
mod ui;

use clap::{Parser, Subcommand};
use config::ProviderConfig;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use ui::{Theme, show};

#[derive(Parser)]
#[command(name = "tryto")]
#[command(about = "Natural language to shell command converter")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Natural language query (when no subcommand is used)
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive setup wizard for configuring providers
    Setup,
    /// Show theme preview for testing
    Theme,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let theme = Theme::default();

    match cli.command {
        Some(Commands::Setup) => {
            if let Err(e) = setup::run_setup(&theme) {
                eprintln!("{}: {}", theme.error("setup failed"), e);
                std::process::exit(1);
            }
        }
        Some(Commands::Theme) => {
            ui::show_theme_preview(&theme);
        }
        None => {
            // Default behavior: generate command from natural language
            if cli.query.is_empty() {
                eprintln!(
                    "{}: tryto <natural language description>",
                    theme.header("usage")
                );
                eprintln!(
                    "{}: tryto list files modified in the last 24 hours",
                    theme.hint("example")
                );
                eprintln!("\n{}: tryto setup", theme.hint("or run setup wizard"));
                std::process::exit(1);
            }

            let query = cli.query.join(" ");
            run_generate(&theme, &query).await;
        }
    }
}

async fn run_generate(theme: &Theme, query: &str) {
    // Load configuration
    let config = match config::Config::load_default() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}: {}", theme.error("error"), e);
            eprintln!(
                "{}: make sure ~/.config/tryto/config.toml exists",
                theme.hint("hint")
            );
            eprintln!("{}: tryto setup", theme.hint("run setup wizard"));
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
    let resp = match generate_command(provider_config, query).await {
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

    // Show the response using the ui module
    show(theme, &resp);
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

/// Create a spinner for long-running operations
fn create_spinner(msg: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Generate command using the provider from configuration
async fn generate_command(
    provider_config: &ProviderConfig,
    query: &str,
) -> Result<response::Response, Box<dyn std::error::Error>> {
    let spinner = create_spinner("Generating command...");

    // Initialize provider from config
    let provider = match providers::init(provider_config) {
        Ok(p) => p,
        Err(e) => {
            spinner.finish_and_clear();
            return Err(Box::new(e));
        }
    };

    // Generate response
    let response = match provider.generate(query).await {
        Ok(r) => r,
        Err(e) => {
            spinner.finish_and_clear();
            return Err(Box::new(e));
        }
    };

    spinner.finish_and_clear();
    parse_response(&response)
}
