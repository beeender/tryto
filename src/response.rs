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
    /// Command name (e.g., "ls", "grep")
    pub name: String,
    /// Command arguments
    pub args: Vec<String>,
}

impl Response {
    /// Parse a simple command line into a Response
    /// This is a basic implementation - for production, use a proper shell parser
    pub fn from_command_line(command_line: &str) -> Self {
        let command_line = command_line.trim();

        // Handle piped commands
        if command_line.contains('|') {
            let parts: Vec<&str> = command_line.split('|').collect();
            let pipeline: Vec<Command> = parts
                .iter()
                .map(|part| Command::parse(part.trim()))
                .collect();
            Response {
                command_line: command_line.to_string(),
                pipeline,
            }
        } else {
            let command = Command::parse(command_line);
            Response {
                command_line: command_line.to_string(),
                pipeline: vec![command],
            }
        }
    }
}

impl Command {
    /// Parse a single command string into a Command
    /// Basic implementation using shell-style splitting
    fn parse(command_str: &str) -> Self {
        let parts: Vec<String> = command_str
            .split_whitespace()
            .map(String::from)
            .collect();

        if parts.is_empty() {
            return Command {
                name: String::new(),
                args: Vec::new(),
            };
        }

        Command {
            name: parts[0].clone(),
            args: parts[1..].to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let response = Response::from_command_line("ls -l -a");
        assert_eq!(response.command_line, "ls -l -a");
        assert_eq!(response.pipeline.len(), 1);
        assert_eq!(response.pipeline[0].name, "ls");
        assert_eq!(response.pipeline[0].args, vec!["-l", "-a"]);
    }

    #[test]
    fn test_piped_command() {
        let response = Response::from_command_line("ls -l | grep abc");
        assert_eq!(response.command_line, "ls -l | grep abc");
        assert_eq!(response.pipeline.len(), 2);
        assert_eq!(response.pipeline[0].name, "ls");
        assert_eq!(response.pipeline[0].args, vec!["-l"]);
        assert_eq!(response.pipeline[1].name, "grep");
        assert_eq!(response.pipeline[1].args, vec!["abc"]);
    }
}
