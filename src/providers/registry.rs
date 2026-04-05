//! Provider registry implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API protocol a provider speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderProtocol {
    /// OpenAI Chat Completions API
    Openai,
    /// Anthropic Messages API
    Anthropic,
    /// Ollama API (no API key required)
    Ollama,
}

/// Definition of an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDefinition {
    /// Unique identifier (e.g., "groq", "moonshot")
    pub id: String,
    /// Which API protocol to use
    pub protocol: ProviderProtocol,
    /// Default base URL. If None, uses the provider's default.
    #[serde(default)]
    pub default_base_url: Option<String>,
    /// Default model if none specified
    pub default_model: String,
    /// Available models for this provider
    pub models: Vec<String>,
    /// Human-readable description
    pub description: String,
}

/// Registry validation error.
#[derive(Debug)]
pub enum ValidationError {
    DuplicateId(String),
    EmptyModels(String),
    InvalidDefaultModel { provider: String, model: String },
    EmptyId,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DuplicateId(id) => write!(f, "duplicate provider ID: {}", id),
            ValidationError::EmptyModels(id) => {
                write!(f, "provider '{}' has empty models list", id)
            }
            ValidationError::InvalidDefaultModel { provider, model } => write!(
                f,
                "provider '{}' default model '{}' not in models list",
                provider, model
            ),
            ValidationError::EmptyId => write!(f, "provider ID cannot be empty"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Registry of known LLM providers.
pub struct ProviderRegistry {
    providers: Vec<ProviderDefinition>,
    /// id -> index into providers
    lookup: HashMap<String, usize>,
}

impl ProviderRegistry {
    /// Build a registry from a list of provider definitions.
    pub fn new(providers: Vec<ProviderDefinition>) -> Self {
        let mut lookup = HashMap::new();
        for (idx, def) in providers.iter().enumerate() {
            lookup.insert(def.id.clone(), idx);
        }
        Self { providers, lookup }
    }

    /// Load the default registry from compiled-in providers.toml.
    pub fn load() -> Result<Self, ValidationError> {
        #[derive(Deserialize)]
        struct ProvidersFile {
            provider: Vec<ProviderDefinition>,
        }

        let file: ProvidersFile = toml::from_str(include_str!("./providers.toml"))
            .expect("built-in providers.toml must be valid TOML");

        Self::validate(&file.provider)?;
        Ok(Self::new(file.provider))
    }

    /// Validate provider definitions.
    fn validate(providers: &[ProviderDefinition]) -> Result<(), ValidationError> {
        use std::collections::HashSet;

        let mut seen_ids = HashSet::new();

        for def in providers {
            // Check for empty ID
            if def.id.is_empty() {
                return Err(ValidationError::EmptyId);
            }

            // Check for duplicate IDs
            if !seen_ids.insert(def.id.clone()) {
                return Err(ValidationError::DuplicateId(def.id.clone()));
            }

            // Check for empty models list
            if def.models.is_empty() {
                return Err(ValidationError::EmptyModels(def.id.clone()));
            }

            // Check that default_model exists in models
            if !def.models.contains(&def.default_model) {
                return Err(ValidationError::InvalidDefaultModel {
                    provider: def.id.clone(),
                    model: def.default_model.clone(),
                });
            }
        }

        Ok(())
    }

    /// Find a provider by ID.
    pub fn get(&self, id: &str) -> Option<&ProviderDefinition> {
        self.lookup.get(id).map(|&idx| &self.providers[idx])
    }

    /// Get all providers.
    pub fn all(&self) -> &[ProviderDefinition] {
        &self.providers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_registry() {
        let registry = ProviderRegistry::load().expect("valid providers.toml");
        assert!(registry.get("openai").is_some());
        assert!(registry.get("anthropic").is_some());
        assert!(registry.get("moonshot").is_some());
    }

    #[test]
    fn test_provider_properties() {
        let registry = ProviderRegistry::load().expect("valid providers.toml");
        let openai = registry.get("openai").unwrap();
        assert_eq!(openai.protocol, ProviderProtocol::Openai);
        assert_eq!(openai.default_model, "gpt-4o");
        assert!(!openai.models.is_empty());
    }
}
