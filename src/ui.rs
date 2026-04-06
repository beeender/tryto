pub mod theme;

pub use theme::Theme;

use crate::response::Response;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Example response JSON for theme demonstration
pub const EXAMPLE_RESPONSE_JSON: &str = include_str!("ui/example_response.json");

/// Parse the example response from JSON
pub fn example_response() -> Response {
    serde_json::from_str(EXAMPLE_RESPONSE_JSON).expect("failed to parse example_response.json")
}

/// Display a response with the given theme
pub fn show(theme: &Theme, response: &Response) {
    // Show the pipeline info with descriptions
    response.pipeline.iter().for_each(|cmd| {
        println!(
            "{} - {}",
            theme.executable(&cmd.executable),
            theme.description(&cmd.description)
        );
        for arg in &cmd.args {
            println!(
                "  {} {}",
                theme.argument(format!("{:<12}", arg.name)),
                theme.description(&arg.description)
            );
        }
        println!();
    });
    println!("$ {}", theme.command_line(&response.command_line));
    print!("\n{} ", theme.prompt("Execute? [Y/n]"));
}

/// Show theme preview for testing
pub fn show_theme_preview(theme: &Theme) {
    println!("{}", theme.header("Theme Preview"));
    println!();

    println!("{}", theme.description("Semantic color samples:"));
    println!("  {} - header text", theme.header("header"));
    println!("  {} - description text", theme.description("description"));
    println!(
        "  {} - executable/command names",
        theme.executable("executable")
    );
    println!(
        "  {} - command line examples",
        theme.command_line("command_line")
    );
    println!("  {} - arguments/flags", theme.argument("argument"));
    println!("  {} - error messages", theme.error("error"));
    println!("  {} - warning messages", theme.warning("warning"));
    println!("  {} - hints/tips", theme.hint("hint"));
    println!("  {} - step numbers", theme.step_number(1));
    println!("  {} - prompts", theme.prompt("prompt>"));
    println!();

    println!("{}", theme.header("Example Command Output"));
    println!();

    let example_resp = example_response();
    show(theme, &example_resp);
    println!();
}

/// Create a spinner for long-running operations
pub fn create_spinner(msg: &str) -> ProgressBar {
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
