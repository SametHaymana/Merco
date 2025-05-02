use crate::task::task::Task;
use merco_llmproxy::{
    ChatMessage, CompletionKind, CompletionRequest, LlmConfig, LlmProvider, Tool,
    execute_tool, get_provider, traits::ChatMessageRole,
};
use std::sync::Arc;

pub struct AgentLLMConfig {
    base_config: LlmConfig,
    model_name: String,
    temperature: f32,
    max_tokens: u32,
}

impl AgentLLMConfig {
    pub fn new(
        base_config: LlmConfig,
        model_name: String,
        temperature: f32,
        max_tokens: u32,
    ) -> Self {
        Self {
            base_config,
            model_name,
            temperature,
            max_tokens,
        }
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
    pub fn new(
        llm_config: AgentLLMConfig,
        backstory: String,
        goals: Vec<String>,
        tools: Vec<Tool>,
    ) -> Self {
        let provider = get_provider(llm_config.base_config.clone()).unwrap();
        Self {
            llm_config,
            backstory,
            goals,
            tools,
            provider,
        }
    }

    pub async fn call(&self, task: Task) -> Result<String, String> {
        let mut messages = vec![
            ChatMessage::new(
                ChatMessageRole::System,
                Some(self.backstory.clone()),
                None,
                None,
            ),
            ChatMessage::new(
                ChatMessageRole::User,
                Some(self.goals.clone().join("\n")),
                None,
                None,
            ),
            ChatMessage::new(
                ChatMessageRole::User,
                Some(format!(
                    "TASK: {}\nEXPECTED OUTPUT: {}",
                    task.description,
                    task.expected_output.unwrap_or("None".to_string())
                )),
                None,
                None,
            ),
        ];

        loop {
            let request = CompletionRequest::new(
                messages.clone(),
                self.llm_config.model_name.clone(),
                Some(self.llm_config.temperature),
                Some(self.llm_config.max_tokens),
                Some(self.tools.clone()),
            );

            match self.provider.completion(request).await {
                Ok(response) => {
                    match response.kind {
                        CompletionKind::Message { content } => {
                            return Ok(content);
                        }
                        CompletionKind::ToolCall { tool_calls } => {
                            let assistant_message = ChatMessage {
                                role: ChatMessageRole::Assistant,
                                content: None,
                                tool_call_id: None,
                                tool_calls: Some(tool_calls.clone()),
                            };
                            messages.push(assistant_message);

                            for call in tool_calls {
                                let tool_result_content = match execute_tool(&call.function.name, &call.function.arguments) {
                                    Ok(result) => result,
                                    Err(e) => {
                                        eprintln!("Tool Execution Error: {}", e);
                                        format!("Error executing tool {}: {}", call.function.name, e)
                                    }
                                };
                                messages.push(ChatMessage::new(
                                    ChatMessageRole::Tool,
                                    Some(tool_result_content),
                                    None,
                                    Some(call.id),
                                ));
                            }
                        }
                    }
                },
                Err(e) => return Err(e.to_string()),
            }
        }
    }
}
