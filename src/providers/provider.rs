use rig::client::CompletionClient;
use rig::completion::Prompt;

use crate::config::ProviderConfig;
use crate::prompts;
use crate::providers::{Error, ProviderInner, ProviderProtocol, select};

/// Anthropic requires max_token
/// - 1024 is generous; most responses will be much smaller
const MAX_TOKENS: u64 = 1024;

/// Provider instance - configured and ready to use.
pub struct Provider {
    pub(crate) inner: ProviderInner,
    pub(crate) model: String,
}

impl Provider {
    /// Initialize a provider from configuration.
    pub fn init(config: &ProviderConfig) -> Result<Self, Error> {
        let def = select(&config.provider)
            .ok_or_else(|| Error::UnknownProvider(config.provider.clone()))?;

        let inner = match def.protocol {
            ProviderProtocol::Openai => {
                use rig::providers::openai;

                let api_key = config.api_key.as_deref().ok_or(Error::MissingApiKey)?;

                let mut builder = openai::Client::builder().api_key(api_key);

                if let Some(base_url) = &config.base_url {
                    builder = builder.base_url(base_url);
                } else if let Some(default_url) = &def.default_base_url {
                    builder = builder.base_url(default_url);
                }

                ProviderInner::OpenAi(builder.build()?)
            }
            ProviderProtocol::Anthropic => {
                use rig::providers::anthropic;

                let api_key = config.api_key.as_deref().ok_or(Error::MissingApiKey)?;

                let mut builder = anthropic::Client::builder().api_key(api_key);

                if let Some(base_url) = &config.base_url {
                    builder = builder.base_url(base_url);
                } else if let Some(default_url) = &def.default_base_url {
                    builder = builder.base_url(default_url);
                }

                ProviderInner::Anthropic(builder.build()?)
            }
            ProviderProtocol::Ollama => {
                use rig::client::Nothing;
                use rig::providers::ollama;

                ProviderInner::Ollama(ollama::Client::new(Nothing)?)
            }
        };

        Ok(Self {
            inner,
            model: config.default_model.clone(),
        })
    }

    /// Generate a response for the given query.
    pub async fn generate(&self, query: &str) -> Result<String, Error> {
        let preamble = prompts::build_prompt();
        log::debug!("Prompt:\n{}\nQuery: {}", preamble, query);
        let response = match &self.inner {
            ProviderInner::OpenAi(client) => {
                let agent = client
                    .agent(&self.model)
                    .preamble(&preamble)
                    .max_tokens(MAX_TOKENS)
                    .build();
                agent.prompt(query).await?
            }
            ProviderInner::Anthropic(client) => {
                let agent = client
                    .agent(&self.model)
                    .preamble(&preamble)
                    .max_tokens(MAX_TOKENS)
                    .build();
                agent.prompt(query).await?
            }
            ProviderInner::Ollama(client) => {
                let agent = client.agent(&self.model).preamble(&preamble).build();
                agent.prompt(query).await?
            }
        };

        log::debug!("Response:\n{}", response);
        Ok(response)
    }
}
