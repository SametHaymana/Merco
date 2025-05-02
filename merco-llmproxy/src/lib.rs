pub mod config;
pub mod providers;
pub mod traits;

pub use config::{ConfigError, LlmConfig, Provider};
pub use providers::{OllamaProvider, OpenAIProvider};
pub use traits::{
    ChatMessage, CompletionRequest, CompletionResponse, CompletionStream, CompletionStreamChunk,
    LlmProvider, ProviderError, TokenUsage,
};

// Optional: A factory function to create a provider instance based on config
use std::sync::Arc;

pub fn get_provider(config: LlmConfig) -> Result<Arc<dyn LlmProvider>, ProviderError> {
    config.validate().map_err(|e| ProviderError::ConfigError(e.to_string()))?;

    match config.provider {
        Provider::OpenAI => Ok(Arc::new(OpenAIProvider::new(config))),
        Provider::Ollama => Ok(Arc::new(OllamaProvider::new(config))),
        Provider::Anthropic => Err(ProviderError::Unsupported("Anthropic provider not yet implemented".to_string())),
        Provider::Custom => Err(ProviderError::Unsupported("Custom provider logic not yet implemented".to_string())),
        // Handle other providers
    }
}
