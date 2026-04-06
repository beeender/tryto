use std::collections::HashSet;
use std::env;
use std::fs;

/// Tool definition from tools.toml - only id is used at runtime
#[derive(Debug, Clone, serde::Deserialize)]
struct Tool {
    id: String,
}

/// Wrapper for TOML deserialization
#[derive(Debug, serde::Deserialize)]
struct ToolsToml {
    tool: Vec<Tool>,
}

/// Tool registry loaded from compiled-in TOML
struct ToolRegistry {
    tools: Vec<Tool>,
}

impl ToolRegistry {
    /// Load the default registry from compiled-in tools.toml
    fn load() -> Self {
        const TOOLS_TOML: &str = include_str!("tools.toml");
        let parsed: ToolsToml = toml::from_str(TOOLS_TOML)
            .expect("failed to parse tools.toml - check for TOML syntax errors");

        // Validate: check for duplicate IDs
        let mut seen = HashSet::new();
        for tool in &parsed.tool {
            if !seen.insert(&tool.id) {
                panic!("duplicate tool ID in tools.toml: {}", tool.id);
            }
        }

        Self { tools: parsed.tool }
    }
}

/// Scan PATH directories and return available modern tools.
pub fn scan_modern_tools() -> Vec<String> {
    let registry = ToolRegistry::load();
    let tool_ids: Vec<String> = registry.tools.iter().map(|t| t.id.clone()).collect();

    let path_dirs: Vec<String> = env::var("PATH")
        .ok()
        .map(|p| p.split(':').map(String::from).collect())
        .unwrap_or_default();

    let mut found = HashSet::new();

    for dir in path_dirs {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                if let Some(name) = file_name.to_str()
                    && tool_ids.iter().any(|id| id.as_str() == name)
                {
                    // Verify it's executable (Unix check)
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(metadata) = entry.metadata() {
                            let mode = metadata.permissions().mode();
                            // Check if any execute bit is set
                            if mode & 0o111 != 0 {
                                found.insert(name.to_string());
                            }
                        }
                    }
                    #[cfg(not(unix))]
                    {
                        found.insert(name.to_string());
                    }
                }
            }
        }
    }

    // Return in consistent order (same as TOML file)
    tool_ids
        .into_iter()
        .filter(|t| found.contains(t))
        .collect()
}

/// Build a summary of available modern tools for the prompt.
pub fn build_tools_context() -> String {
    let tools = scan_modern_tools();

    if tools.is_empty() {
        return String::new();
    }

    let tools_list = tools.join(", ");
    format!(
        "\n## Available Modern Tools\n\n\
        The following modern alternatives are available and preferred when appropriate:\n\
        {}\n\n\
        Prefer these over traditional tools (e.g., use 'fd' over 'find', 'rg' over 'grep').\n",
        tools_list
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_loads() {
        let registry = ToolRegistry::load();
        assert!(!registry.tools.is_empty());
    }

    #[test]
    fn test_scan_does_not_panic() {
        let _tools = scan_modern_tools();
    }
}
