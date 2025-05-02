
use merco_llmproxy::{execute_tool, get_provider, traits::ChatMessageRole, ChatMessage, CompletionKind, CompletionRequest, LlmConfig, LlmProvider, Tool};
use crate::task::task::Task;
use std::sync::Arc;

pub struct AgentLLMConfig {
    base_config: LlmConfig,
    model_name: String,
    temperature: f32,
    max_tokens: u32,
}


impl AgentLLMConfig {
    pub fn new(base_config: LlmConfig, model_name: String, temperature: f32, max_tokens: u32) -> Self {
        Self { base_config, model_name, temperature, max_tokens }
    }
}


pub struct Agent {
    llm_config: AgentLLMConfig,
    provider: Arc<dyn LlmProvider>,
    pub backstory: String,
    pub goals: Vec<String>,
    pub tools: Vec<Tool>,
}

impl Agent {
    pub fn new(llm_config: AgentLLMConfig, backstory: String, goals: Vec<String>, tools: Vec<Tool>) -> Self {
        let provider = get_provider(llm_config.base_config.clone()).unwrap();
        Self { llm_config, backstory, goals, tools, provider }
    }

    pub async fn call(&self, task: Task) -> Result<String, String> {
        let messages = vec![
            ChatMessage::new(ChatMessageRole::System, Some(self.backstory.clone()), None, None),
            ChatMessage::new(ChatMessageRole::User, Some(self.goals.clone().join("\n")), None, None),
            ChatMessage::new(ChatMessageRole::User, Some(format!("EXPECTED OUTPUT: {}", task.expected_output.unwrap_or("None".to_string()))), None, None),
            ChatMessage::new(ChatMessageRole::User, Some(task.description), None, None),
        ];

        let request = CompletionRequest::new(messages, self.llm_config.model_name.clone(), Some(self.llm_config.temperature), Some(self.llm_config.max_tokens), Some(self.tools.clone()));

        match self.provider.completion(request).await {
            Ok(response) => {
                match response.kind {
                    CompletionKind::Message { content } => {
                        Ok(content)
                    }
                    CompletionKind::ToolCall { tool_calls } => {
                        let mut results = vec![];
                        for call in tool_calls {
                            match execute_tool(&call.function.name, &call.function.arguments) {
                                Ok(result) => results.push(result),
                                Err(e) => println!("Execution Error: {}", e),
                            }
                        }
                        Ok(results.join("\n"))
                    }
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        }

    }
}

