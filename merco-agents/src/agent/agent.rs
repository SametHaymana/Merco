
use rllm::chat::{ChatMessage, StructuredOutputFormat, Tool};

use crate::llm::interface::{LLM, LLMConfig};
use crate::task::task::Task;

pub struct Agent {
    pub llm_config: LLMConfig,
    pub backstory: String,
    pub goals: Vec<String>,
    pub tools: Vec<Tool>,
}

impl Agent {
    pub fn new(llm_config: LLMConfig, backstory: String, goals: Vec<String>, tools: Vec<Tool>) -> Self {
        Self { llm_config, backstory, goals, tools }
    }

    pub async fn run(&self, task: Task) -> Result<String, String> {
        let messages = vec![
            ChatMessage::assistant().content(self.backstory.clone()).build(),
            ChatMessage::user().content(self.goals.clone().join("\n")).build(),
            ChatMessage::user().content(task.description).build(),
        ];

        let llm = LLM::new(self.llm_config.clone(), task.expected_output);

        let response = llm.provider.chat(&messages).await
        .map_err(|e| e.to_string())?;

        Ok(response.text().unwrap())
    }
}

