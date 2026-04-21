mod config;
mod prompts;
mod providers;
mod response;
mod setup;
mod ui;

use clap::{Parser, Subcommand};
use config::ProviderConfig;
use ui::Theme;

#[derive(Parser)]
#[command(name = "tryto")]
#[command(about = "Natural language to shell command converter")]
#[command(
    after_help = "Examples:\n  tryto list files modified in the last 24 hours\n  tryto find all \
                  python files\n  tryto show git log with graph"
)]
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
    /// Manage themes
    Theme {
        #[command(subcommand)]
        command: ThemeCommands,
    },
}

#[derive(Subcommand)]
enum ThemeCommands {
    /// List available themes
    List,
    /// Preview a theme (defaults to current)
    Preview { name: Option<String> },
    /// Set theme as default
    Set { name: String },
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    // Load theme from config or use default
    let config = config::Config::load_default().ok();
    let theme_name = config.as_ref().and_then(|c| c.theme.as_deref());
    let theme = ui::theme::load(theme_name).unwrap_or_else(|e| {
        log::error!("failed to load theme: {}, fallback to default", e);
        Theme::default()
    });

    match cli.command {
        Some(Commands::Setup) => {
            if let Err(e) = setup::run_setup(&theme) {
                eprintln!("{}: {}", theme.error("setup failed"), e);
                std::process::exit(1);
            }
        }
        Some(Commands::Theme { command }) => {
            handle_theme_command(&theme, command, config);
        }
        None => {
            if cli.query.is_empty() {
                <Cli as clap::CommandFactory>::command()
                    .print_help()
                    .unwrap();
                println!();
                std::process::exit(1);
            }
            let query = cli.query.join(" ");
            run_generate(&theme, &query).await;
        }
    }
}

fn handle_theme_command(theme: &Theme, command: ThemeCommands, config: Option<config::Config>) {
    use ui::theme;

    match command {
        ThemeCommands::List => {
            println!("{}", theme.header("Available themes:"));
            println!();
            for name in theme::list_presets() {
                let current = config.as_ref().and_then(|c| c.theme.as_deref());
                let display_name = if Some(name) == current {
                    format!("* {}", name)
                } else {
                    format!("  {}", name)
                };
                println!("{}", theme.executable(display_name));
            }
            println!();
            println!(
                "{}",
                theme.hint("Use 'tryto theme preview <name>' to preview a theme")
            );
            println!(
                "{}",
                theme.hint("Use 'tryto theme set <name>' to set as default")
            );
        }
        ThemeCommands::Preview { name } => {
            let preview_theme = match name.as_deref() {
                Some(n) => match theme::from_preset(n) {
                    Ok(t) => {
                        println!("{}", theme.header(format!("Previewing theme: {}", n)));
                        println!();
                        t
                    }
                    Err(e) => {
                        log::error!("failed to load theme '{}': {}", n, e);
                        std::process::exit(1);
                    }
                },
                None => theme.clone(),
            };
            ui::show_theme_preview(&preview_theme);
        }
        ThemeCommands::Set { name } => {
            if theme::get_preset(&name).is_none() {
                eprintln!("{}: theme '{}' not found", theme.error("error"), name);
                eprintln!(
                    "{}",
                    theme.hint("Run 'tryto theme list' to see available themes")
                );
                std::process::exit(1);
            }
            let mut cfg = config.unwrap_or_else(|| {
                eprintln!(
                    "{}: no config found, run 'tryto setup' first",
                    theme.error("error")
                );
                std::process::exit(1);
            });
            cfg.theme = Some(name.clone());
            if let Err(e) = cfg.save_default() {
                eprintln!("{}: failed to save config: {}", theme.error("error"), e);
                std::process::exit(1);
            }
            println!(
                "{} Theme set to '{}'",
                theme.hint("✓"),
                theme.executable(&name)
            );
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

    // Show the response and get confirmation type
    let confirmation = ui::show(theme, &resp);
    use std::io::Write;
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let should_execute = confirmation.check(&input);

    if should_execute {
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

/// Generate command using the provider from configuration
async fn generate_command(
    provider_config: &ProviderConfig,
    query: &str,
) -> Result<response::Response, Box<dyn std::error::Error>> {
    // Initialize provider from config
    let provider = match providers::init(provider_config) {
        Ok(p) => p,
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    let spinner = ui::create_spinner("Generating command...");
    // Generate response
    let response = match provider.generate(query).await {
        Ok(r) => r,
        Err(e) => {
            spinner.finish_and_clear();
            return Err(Box::new(e));
        }
    };

    spinner.finish_and_clear();
    response::Response::parse(&response).map_err(|e| e.into())
}
