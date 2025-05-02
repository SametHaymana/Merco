use rllm::chat::StructuredOutputFormat;

pub struct Task {
    pub description: String,
    pub expected_output: Option<StructuredOutputFormat>,
}

impl Task {
    pub fn new(description: String, expected_output: Option<StructuredOutputFormat>) -> Self {
        Self { description, expected_output }
    }
}

