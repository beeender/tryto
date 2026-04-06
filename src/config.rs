use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Default provider to use when none is specified
    pub default_provider: String,
    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,
}

/// Provider configuration - flat structure works for most providers
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    /// Provider type (openai, anthropic, deepseek, gemini, ollama, xai,
    /// perplexity, groq)
    pub provider: String,
    /// API key (optional for some providers like ollama)
    pub api_key: Option<String>,
    /// Base URL (optional, uses provider default if not set)
    pub base_url: Option<String>,
    /// Default model to use
    pub default_model: String,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from default location (~/.config/tryto/config.toml)
    pub fn load_default() -> Result<Self, ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::HomeDirNotFound)?;
        let config_path = home.join(".config").join("tryto").join("config.toml");
        Self::from_file(config_path)
    }

    /// Get a provider configuration by name
    #[allow(dead_code)]
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    /// Get the default provider configuration
    pub fn get_default_provider(&self) -> Option<&ProviderConfig> {
        self.providers.get(&self.default_provider)
    }

    /// Save configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let toml_str = toml::to_string_pretty(self)?;
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, toml_str)?;
        Ok(())
    }

    /// Save configuration to default location (~/.config/tryto/config.toml)
    pub fn save_default(&self) -> Result<(), ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::HomeDirNotFound)?;
        let config_path = home.join(".config").join("tryto").join("config.toml");
        self.save_to_file(config_path)
    }
}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    HomeDirNotFound,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::TomlDe(e) => write!(f, "TOML parse error: {}", e),
            ConfigError::TomlSer(e) => write!(f, "TOML serialize error: {}", e),
            ConfigError::HomeDirNotFound => write!(f, "Could not find home directory"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::TomlDe(e) => Some(e),
            ConfigError::TomlSer(e) => Some(e),
            ConfigError::HomeDirNotFound => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::TomlDe(e)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        ConfigError::TomlSer(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
            default_provider = "openai"

            [providers.openai]
            provider = "openai"
            api_key = "sk-test123"
            default_model = "gpt-4"

            [providers.anthropic]
            provider = "anthropic"
            api_key = "sk-ant-test123"
            default_model = "claude-3-opus-20240229"

            [providers.ollama]
            provider = "ollama"
            base_url = "http://localhost:11434"
            default_model = "llama3"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default_provider, "openai");

        let openai = config.get_provider("openai").unwrap();
        assert_eq!(openai.provider, "openai");
        assert_eq!(openai.api_key, Some("sk-test123".to_string()));
        assert_eq!(openai.default_model, "gpt-4");

        let anthropic = config.get_provider("anthropic").unwrap();
        assert_eq!(anthropic.provider, "anthropic");
        assert_eq!(anthropic.default_model, "claude-3-opus-20240229");

        let ollama = config.get_provider("ollama").unwrap();
        assert_eq!(ollama.provider, "ollama");
        assert_eq!(ollama.api_key, None);
        assert_eq!(ollama.base_url, Some("http://localhost:11434".to_string()));
    }

    #[test]
    fn test_get_default_provider() {
        let toml_str = r#"
            default_provider = "deepseek"

            [providers.deepseek]
            provider = "deepseek"
            api_key = "sk-deepseek"
            default_model = "deepseek-chat"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        let default = config.get_default_provider().unwrap();
        assert_eq!(default.provider, "deepseek");
    }
}
