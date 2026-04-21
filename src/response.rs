use serde::{Deserialize, Serialize};

/// Structured command response from AI
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    /// The full command line as a single string
    pub command_line: String,
    /// Pipeline of commands (for piped commands)
    pub pipeline: Vec<Command>,
    /// Danger level: 0=safe, 1=caution, 2=dangerous, 3=critical
    #[serde(default)]
    pub dangerous_level: u8,
    /// Explanation of why the command is dangerous (for level 1-3)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dangerous_reason: Option<String>,
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

impl Response {
    /// Parse a response from JSON string, handling markdown code blocks
    pub fn parse(response: &str) -> Result<Self, serde_json::Error> {
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

        serde_json::from_str(&json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json = r#"{"command_line": "ls -la", "pipeline": [{"executable": "ls", "description": "List files", "args": [{"name": "-la", "description": "All files, long format"}]}]}"#;
        let resp = Response::parse(json).unwrap();
        assert_eq!(resp.command_line, "ls -la");
        assert_eq!(resp.pipeline.len(), 1);
        assert_eq!(resp.pipeline[0].executable, "ls");
        assert_eq!(resp.pipeline[0].args.len(), 1);
        assert_eq!(resp.pipeline[0].args[0].name, "-la");
    }

    #[test]
    fn test_parse_markdown_code_block() {
        let json = r#"```json
{"command_line": "cat file.txt", "pipeline": [{"executable": "cat", "description": "Show file", "args": [{"name": "file.txt", "description": "File to display"}]}]}
```"#;
        let resp = Response::parse(json).unwrap();
        assert_eq!(resp.command_line, "cat file.txt");
        assert_eq!(resp.pipeline[0].executable, "cat");
    }

    #[test]
    fn test_parse_empty_args() {
        let json = r#"{"command_line": "pwd", "pipeline": [{"executable": "pwd", "description": "Print working directory", "args": []}]}"#;
        let resp = Response::parse(json).unwrap();
        assert_eq!(resp.command_line, "pwd");
        assert!(resp.pipeline[0].args.is_empty());
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"not valid json"#;
        assert!(Response::parse(json).is_err());
    }

    #[test]
    fn test_parse_dangerous_level() {
        let json = r#"{"command_line": "rm -rf /", "pipeline": [{"executable": "rm", "description": "Remove files", "args": [{"name": "-rf", "description": "Recursive force"}]}], "dangerous_level": 3, "dangerous_reason": "Permanently deletes files"}"#;
        let resp = Response::parse(json).unwrap();
        assert_eq!(resp.dangerous_level, 3);
        assert_eq!(
            resp.dangerous_reason,
            Some("Permanently deletes files".to_string())
        );
    }

    #[test]
    fn test_parse_dangerous_level_defaults_to_zero() {
        let json = r#"{"command_line": "ls -la", "pipeline": [{"executable": "ls", "description": "List files", "args": []}]}"#;
        let resp = Response::parse(json).unwrap();
        assert_eq!(resp.dangerous_level, 0);
        assert!(resp.dangerous_reason.is_none());
    }
}
