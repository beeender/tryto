pub mod os_info;

pub const COMMAND_GENERATOR: &str = include_str!("prompts/command_generator.md");

/// Build the full system prompt with platform context.
pub fn build_prompt() -> String {
    format!(
        "{}\n\n{}",
        COMMAND_GENERATOR,
        os_info::build_context()
    )
}
