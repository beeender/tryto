pub mod executables;
pub mod os_info;

pub const COMMAND_GENERATOR: &str = include_str!("prompts/command_generator.md");

/// Build the full system prompt with platform context.
pub fn build_prompt() -> String {
    let platform = os_info::build_context();
    let tools = executables::build_tools_context();

    if tools.is_empty() {
        format!("{}\n\n{}", COMMAND_GENERATOR, platform)
    } else {
        format!("{}\n\n{}\n{}", COMMAND_GENERATOR, platform, tools)
    }
}
