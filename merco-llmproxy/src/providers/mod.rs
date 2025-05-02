// Declare provider implementation modules here
pub mod openai;
pub mod ollama;
// pub mod anthropic; // Add later

// Potentially re-export provider structs if needed
pub use openai::OpenAIProvider;
pub use ollama::OllamaProvider; 