use serde::{Deserialize, Serialize};

/// Structured command response from AI
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    /// The full command line as a single string
    pub command_line: String,
    /// Pipeline of commands (for piped commands)
    pub pipeline: Vec<Command>,
}

/// A single command in the pipeline
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Command {
    /// The executable name (e.g., "ls", "grep")
    pub executable: String,
    /// Description of what this command does
    pub description: String,
    /// Command arguments with their descriptions
    pub args: Vec<Argument>,
}

/// A single argument
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Argument {
    /// The argument value (e.g., "-l", "*.txt")
    pub name: String,
    /// Description of what this argument does
    pub description: String,
}
