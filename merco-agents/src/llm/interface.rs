use serde::{Deserialize, Serialize};
use rllm::{builder::{LLMBackend, LLMBuilder}, chat::{StructuredOutputFormat, Tool}, LLMProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    provider: String,
    model_name: String,
    base_url: String,
    api_key: Option<String>,

}

impl LLMConfig {
    pub fn new(provider: String, model_name: String, api_key: Option<String>, base_url: String) -> Self {
        Self { provider, model_name, api_key, base_url }
    }
}

pub struct LLM {
    pub provider: Box<dyn LLMProvider>,
}


impl LLM {
    pub fn new(config: LLMConfig, schema: Option<impl Into<StructuredOutputFormat>>) -> Self {
        match config.provider.as_str() {
            "ollama" => {
                let mut builder = LLMBuilder::new()
                    .backend(LLMBackend::Ollama)
                    .base_url(config.base_url.clone());

                if let Some(s) = schema {
                    builder = builder.schema(s);
                }

                let provider = builder
                    //.tool_choice(rllm::chat::ToolChoice::Auto) // Assuming Auto is still desired even with schema
                    .build()
                    .expect("Failed to build Ollama LLM");
                Self { provider }
            }
            "openrouter" => {
                let mut builder = LLMBuilder::new()
                    .backend(LLMBackend::OpenAI) // OpenRouter uses OpenAI compatible API
                    .base_url(config.base_url.clone());
                    
                match config.api_key {
                    Some(key) => {
                        builder = builder.api_key(key);
                    }
                    None => {
                        panic!("OpenRouter API key is required");
                    }
                }

                if let Some(s) = schema {
                    builder = builder.schema(s);
                }

                let provider = builder
                    //.tool_choice(rllm::chat::ToolChoice::Auto) // Assuming Auto is still desired even with schema
                    .build()
                    .expect("Failed to build OpenRouter LLM");
                Self { provider }
            }
            _ => panic!("Unsupported provider: {}", config.provider),
        }
    }
}
