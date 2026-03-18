mod config;

use config::{Config, ProviderConfig};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use std::env;

const SYSTEM_PROMPT: &str = r#"You are a command-line assistant. Your task is to convert the user's natural language request into a valid shell command and explain what it does.

Rules:
1. Return the command on the first line (plain text, no markdown)
2. Return a compact explanation in Markdown format starting from the second line
3. The explanation should be concise and include:
   - Command name with brief description
   - Arguments/options as bullet points with brief descriptions
   - For complex expressions (like awk scripts, regex, etc.), explain the key components
4. Use compact Markdown formatting
5. The command should be safe and executable in a standard shell (bash/zsh)
6. If the request is ambiguous, provide the most common interpretation

Format:
<command>
## `<binary>` - <brief description>
- `<arg>` - <brief description>
- `<complex_arg>` - <explain key components>

Examples:
User: "list all files in current directory"
Response:
ls -la
## `ls` - list directory contents
- `-l` - Use long listing format (detailed info)
- `-a` - Do not ignore entries starting with . (show hidden files)

User: "find all python files"
Response:
find . -name "*.py"
## `find` - search for files in directory hierarchy
- `.` - Start searching from current directory
- `-name "*.py"` - Match files ending with .py extension

User: "show git status"
Response:
git status
## `git status` - show working tree status
- Displays staged, unstaged, and untracked files

User: "find smallest file and add 1 to its size"
Response:
ls -lS | awk 'NR==2 {print $5 + 1}'
## Pipeline: `ls` + `awk`
- `ls -lS` - List files in long format, sorted by size (largest first)
- `| awk 'NR==2 {print $5 + 1}'` - Extract size from smallest file and add 1
  - `NR==2` - Select the 2nd line (smallest file after header)
  - `$5` - The 5th field (file size in bytes)
  - `+ 1` - Add 1 to the size value
"#;

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

    // Generate command and explanation using AI
    let (command, explanation, raw_response) = match generate_command(provider_config, &query).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to generate command: {}", e);
            std::process::exit(1);
        }
    };

    // Write raw AI response to tmp log file for debugging
    let log_path = std::path::PathBuf::from("/tmp/tryto_debug.log");
    if let Err(e) = std::fs::write(&log_path, &raw_response) {
        eprintln!("Warning: Failed to write debug log: {}", e);
    } else {
        eprintln!("Debug: Raw AI response written to {}", log_path.display());
    }

    // Show the explanation first (rendered from markdown using termimad), then the command, and ask for confirmation
    let skin = termimad::MadSkin::default();
    let rendered_explanation = skin.term_text(&explanation);
    println!("\n{}", rendered_explanation);
    println!("\n$ {}", command);
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
            .arg(&command)
            .status()
            .expect("Failed to execute command");
        
        std::process::exit(status.code().unwrap_or(1));
    } else {
        println!("Command cancelled");
    }
}

fn parse_response(response: &str) -> (String, String, String) {
    let lines: Vec<&str> = response.trim().lines().collect();
    if lines.len() >= 2 {
        let command = lines[0].trim().to_string();
        let explanation = lines[1..].join("\n").trim().to_string();
        (command, explanation, response.to_string())
    } else if lines.len() == 1 {
        (lines[0].trim().to_string(), "No explanation provided".to_string(), response.to_string())
    } else {
        ("".to_string(), "No response".to_string(), response.to_string())
    }
}

async fn generate_command(
    provider_config: &ProviderConfig,
    query: &str,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
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
                .preamble(SYSTEM_PROMPT)
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
                .preamble(SYSTEM_PROMPT)
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
                .preamble(SYSTEM_PROMPT)
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
                .preamble(SYSTEM_PROMPT)
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
                .preamble(SYSTEM_PROMPT)
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
                .preamble(SYSTEM_PROMPT)
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
                .preamble(SYSTEM_PROMPT)
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
                .preamble(SYSTEM_PROMPT)
                .max_tokens(1024)
                .build();

            agent.prompt(query).await?
        }
        _ => {
            return Err(format!("Unknown provider: {}", provider_type).into());
        }
    };
    
    Ok(parse_response(&response))
}
