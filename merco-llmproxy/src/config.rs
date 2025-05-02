use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    OpenAI,
    Ollama,
    Anthropic,
    // Add other providers here
    Custom, // For self-hosted or less common providers using a base_url
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: Provider,
    pub model: String,
    pub api_key: Option<String>, // Optional as some providers (like Ollama local) might not need it
    pub base_url: Option<String>, // Optional, mainly for 'Custom' or overriding default URLs
    // Add other common configuration options like timeout, temperature, etc. later
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing API key for provider: {0:?}")]
    MissingApiKey(Provider),
    #[error("Missing base URL for custom provider")]
    MissingBaseUrl,
    // Add other potential configuration errors
}

impl LlmConfig {
    // Basic constructor
    pub fn new(provider: Provider, model: String) -> Self {
        LlmConfig {
            provider,
            model,
            api_key: None,
            base_url: None,
        }
    }

    // Builder-style methods for setting optional fields
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    // Validate the configuration based on the provider
    pub fn validate(&self) -> Result<(), ConfigError> {
        match self.provider {
            Provider::OpenAI | Provider::Anthropic => {
                if self.api_key.is_none() {
                    return Err(ConfigError::MissingApiKey(self.provider.clone()));
                }
            }
            Provider::Custom => {
                if self.base_url.is_none() {
                    return Err(ConfigError::MissingBaseUrl);
                }
                // Custom might still require an API key depending on the specific setup
                 if self.api_key.is_none() {
                     // Or maybe log a warning instead of erroring? Depends on desired behavior.
                     // println!("Warning: Custom provider selected without an API key.");
                 }
            }
            Provider::Ollama => {
                // Ollama typically runs locally and might not need an API key,
                // but might need a base_url if not default localhost.
                // Validation logic can be added here if needed.
            }
        }
        Ok(())
    }
} 