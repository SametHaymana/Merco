#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Task {
    pub description: String,
    pub expected_output: Option<String>,
}

impl Task {
    pub fn new(description: String, expected_output: Option<String>) -> Self {
        Self {
            description,
            expected_output,
        }
    }
}
