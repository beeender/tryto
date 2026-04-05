//! Declarative LLM provider system.
//!
//! Providers are defined in `providers.toml` (compiled-in defaults) so adding
//! a new provider requires zero Rust code changes.
//!
//! # Example
//!
//! ```rust
//! use tryto::providers::{select, init, list};
//!
//! // List all available providers
//! for provider in list() {
//!     println!("{}: {}", provider.id, provider.description);
//! }
//!
//! // Select a provider definition
//! let def = select("moonshot").expect("provider exists");
//!
//! // Initialize a provider from config (ready to use)
//! let provider = init(&provider_config)?;
//! let response = provider.generate("list files").await?;
//! ```

pub mod provider;
pub mod registry;

pub use provider::Provider;
pub use registry::{ProviderDefinition, ProviderProtocol, ProviderRegistry};

use crate::config::ProviderConfig;

/// Provider error type.
#[derive(Debug)]
pub enum Error {
    UnknownProvider(String),
    MissingApiKey,
    Client(String),
    Prompt(rig::completion::PromptError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownProvider(id) => write!(f, "Unknown provider: {}", id),
            Error::MissingApiKey => write!(f, "API key is required for this provider"),
            Error::Client(msg) => write!(f, "Client error: {}", msg),
            Error::Prompt(e) => write!(f, "Prompt error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Prompt(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rig::completion::PromptError> for Error {
    fn from(e: rig::completion::PromptError) -> Self {
        Error::Prompt(e)
    }
}

impl From<rig::http_client::Error> for Error {
    fn from(e: rig::http_client::Error) -> Self {
        Error::Client(e.to_string())
    }
}

/// Internal enum for provider client types.
pub(crate) enum ProviderInner {
    OpenAi(rig::providers::openai::Client),
    Anthropic(rig::providers::anthropic::Client),
    Ollama(rig::providers::ollama::Client),
}

/// Global singleton registry.
static REGISTRY: std::sync::OnceLock<ProviderRegistry> = std::sync::OnceLock::new();

fn get_registry() -> &'static ProviderRegistry {
    REGISTRY.get_or_init(|| {
        ProviderRegistry::load()
            .expect("failed to load provider registry - check providers.toml for errors")
    })
}

/// List all available providers.
pub fn list() -> &'static [ProviderDefinition] {
    get_registry().all()
}

/// Select a provider definition by ID.
pub fn select(id: &str) -> Option<&'static ProviderDefinition> {
    get_registry().get(id)
}

/// Initialize a provider from configuration.
pub fn init(config: &ProviderConfig) -> Result<Provider, Error> {
    Provider::init(config)
}
