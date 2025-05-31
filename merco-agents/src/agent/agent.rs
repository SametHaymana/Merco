use crate::task::task::Task;
use merco_llmproxy::{
    ChatMessage, CompletionKind, CompletionRequest, LlmConfig, LlmProvider, Tool,
    execute_tool, get_provider, traits::ChatMessageRole,
};
use std::sync::Arc;
use std::fmt;

#[derive(Debug, Clone)]
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

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Agent")
         .field("llm_config", &self.llm_config)
         .field("provider", &"<LlmProvider>")
         .field("backstory", &self.backstory)
         .field("goals", &self.goals)
         .field("tools", &self.tools)
         .finish()
    }
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
        const MAX_RETRIES: usize = 3;
        
        for attempt in 1..=MAX_RETRIES {
            println!("Agent execution attempt {} of {}", attempt, MAX_RETRIES);
            
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
                        "TASK: {}\n\nEXPECTED OUTPUT: {}\n\nOUTPUT FORMAT:\n{}",
                        task.description,
                        task.expected_output.as_ref().unwrap_or(&"None".to_string()),
                        task.get_format_prompt() // Include format prompt
                    )),
                    None,
                    None,
                ),
            ];

            // Execute the task with the LLM (existing loop logic)
            let raw_result = match self.execute_with_llm(&mut messages).await {
                Ok(result) => result,
                Err(e) => {
                    if attempt == MAX_RETRIES {
                        return Err(format!("LLM execution failed after {} attempts: {}", MAX_RETRIES, e));
                    }
                    println!("LLM execution failed on attempt {}: {}. Retrying...", attempt, e);
                    continue;
                }
            };

            // Validate the output
            match task.validate_output(&raw_result) {
                Ok(()) => {
                    println!("Output validation successful on attempt {}", attempt);
                    return Ok(raw_result);
                }
                Err(validation_error) => {
                    if attempt == MAX_RETRIES {
                        return Err(format!(
                            "Output validation failed after {} attempts. Last error: {}. Raw output: {}",
                            MAX_RETRIES, validation_error, raw_result
                        ));
                    }
                    println!(
                        "Output validation failed on attempt {}: {}. Retrying...", 
                        attempt, validation_error
                    );
                    
                    // Add feedback message for retry
                    messages.push(ChatMessage::new(
                        ChatMessageRole::User,
                        Some(format!(
                            "Your previous response was invalid: {}. Please provide a corrected response that follows the required format exactly.",
                            validation_error
                        )),
                        None,
                        None,
                    ));
                }
            }
        }
        
        Err("Maximum retry attempts exceeded".to_string())
    }

    // Extracted LLM execution logic (the original loop from call method)
    async fn execute_with_llm(&self, messages: &mut Vec<ChatMessage>) -> Result<String, String> {
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
                            messages.push(ChatMessage::new(
                                ChatMessageRole::Assistant,
                                None,
                                Some(tool_calls.clone()),
                                None,
                            ));
                            
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
