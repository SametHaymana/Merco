use serde_json::Value;
use anyhow::{Result, anyhow};

// Enum to define different output format types
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OutputFormat {
    Text, // Free-form text output
    Json {
        schema: JsonSchema,
        strict: bool, // Whether to enforce strict validation (all fields required)
    },
}

// JSON Schema definition for validation
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsonSchema {
    pub required_fields: Vec<JsonField>,
    pub optional_fields: Vec<JsonField>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsonField {
    pub name: String,
    pub field_type: JsonFieldType,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum JsonFieldType {
    String,
    Number,
    Boolean,
    Array(Box<JsonFieldType>), // Array of specific type
    Object, // Nested object (simplified for now)
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Task {
    pub description: String,
    pub expected_output: Option<String>,
    pub output_format: OutputFormat, // New field for typed output
}

impl Task {
    pub fn new(description: String, expected_output: Option<String>) -> Self {
        Self {
            description,
            expected_output,
            output_format: OutputFormat::Text, // Default to text
        }
    }

    // Constructor for JSON output format
    pub fn new_with_json_output(
        description: String,
        expected_output: Option<String>,
        required_fields: Vec<JsonField>,
        optional_fields: Vec<JsonField>,
        strict: bool,
    ) -> Self {
        Self {
            description,
            expected_output,
            output_format: OutputFormat::Json {
                schema: JsonSchema {
                    required_fields,
                    optional_fields,
                },
                strict,
            },
        }
    }

    // Helper to create a simple JSON task with just field names and types
    pub fn new_simple_json(
        description: String,
        expected_output: Option<String>,
        required_fields: Vec<(String, JsonFieldType)>,
        strict: bool,
    ) -> Self {
        let fields = required_fields
            .into_iter()
            .map(|(name, field_type)| JsonField {
                name,
                field_type,
                description: None,
            })
            .collect();

        Self::new_with_json_output(description, expected_output, fields, vec![], strict)
    }

    // Validate agent output against the expected format
    pub fn validate_output(&self, output: &str) -> Result<()> {
        match &self.output_format {
            OutputFormat::Text => {
                // For text format, any non-empty string is valid
                if output.trim().is_empty() {
                    return Err(anyhow!("Output is empty"));
                }
                Ok(())
            }
            OutputFormat::Json { schema, strict } => {
                self.validate_json_output(output, schema, *strict)
            }
        }
    }

    // JSON-specific validation
    fn validate_json_output(&self, output: &str, schema: &JsonSchema, strict: bool) -> Result<()> {
        // Parse the output as JSON
        let parsed: Value = serde_json::from_str(output.trim())
            .map_err(|e| anyhow!("Output is not valid JSON: {}", e))?;

        // Ensure it's a JSON object
        let obj = parsed.as_object()
            .ok_or_else(|| anyhow!("JSON output must be an object, got: {}", parsed))?;

        // Validate required fields
        for field in &schema.required_fields {
            if !obj.contains_key(&field.name) {
                return Err(anyhow!("Missing required field: '{}'", field.name));
            }

            let value = &obj[&field.name];
            self.validate_field_type(value, &field.field_type, &field.name)?;
        }

        // Validate optional fields (if present)
        for field in &schema.optional_fields {
            if let Some(value) = obj.get(&field.name) {
                self.validate_field_type(value, &field.field_type, &field.name)?;
            }
        }

        // In strict mode, ensure no extra fields are present
        if strict {
            let expected_fields: std::collections::HashSet<&String> = schema
                .required_fields
                .iter()
                .chain(schema.optional_fields.iter())
                .map(|f| &f.name)
                .collect();

            for key in obj.keys() {
                if !expected_fields.contains(key) {
                    return Err(anyhow!("Unexpected field in strict mode: '{}'", key));
                }
            }
        }

        Ok(())
    }

    // Validate individual field types
    fn validate_field_type(&self, value: &Value, expected_type: &JsonFieldType, field_name: &str) -> Result<()> {
        match expected_type {
            JsonFieldType::String => {
                if !value.is_string() {
                    return Err(anyhow!("Field '{}' must be a string, got: {}", field_name, value));
                }
            }
            JsonFieldType::Number => {
                if !value.is_number() {
                    return Err(anyhow!("Field '{}' must be a number, got: {}", field_name, value));
                }
            }
            JsonFieldType::Boolean => {
                if !value.is_boolean() {
                    return Err(anyhow!("Field '{}' must be a boolean, got: {}", field_name, value));
                }
            }
            JsonFieldType::Array(element_type) => {
                let arr = value.as_array()
                    .ok_or_else(|| anyhow!("Field '{}' must be an array, got: {}", field_name, value))?;
                
                // Validate each element in the array
                for (i, element) in arr.iter().enumerate() {
                    self.validate_field_type(element, element_type, &format!("{}[{}]", field_name, i))?;
                }
            }
            JsonFieldType::Object => {
                if !value.is_object() {
                    return Err(anyhow!("Field '{}' must be an object, got: {}", field_name, value));
                }
                // For now, we just check it's an object. Could extend to nested schema validation.
            }
        }
        Ok(())
    }

    // Generate a prompt section describing the expected output format
    pub fn get_format_prompt(&self) -> String {
        match &self.output_format {
            OutputFormat::Text => {
                "Provide your response as plain text.".to_string()
            }
            OutputFormat::Json { schema, strict } => {
                let mut prompt = "You must respond with valid JSON in the following format:\n\n".to_string();
                
                prompt.push_str("{\n");
                
                // Add required fields
                for field in &schema.required_fields {
                    prompt.push_str(&format!(
                        "  \"{}\": <{}>{},  // REQUIRED{}\n", 
                        field.name,
                        self.type_to_string(&field.field_type),
                        if schema.required_fields.last() == Some(field) && schema.optional_fields.is_empty() { "" } else { "," },
                        field.description.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default()
                    ));
                }
                
                // Add optional fields
                for field in &schema.optional_fields {
                    prompt.push_str(&format!(
                        "  \"{}\": <{}>{},  // OPTIONAL{}\n", 
                        field.name,
                        self.type_to_string(&field.field_type),
                        if schema.optional_fields.last() == Some(field) { "" } else { "," },
                        field.description.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default()
                    ));
                }
                
                prompt.push_str("}\n\n");
                
                if *strict {
                    prompt.push_str("IMPORTANT: Only include the specified fields. No additional fields are allowed.\n");
                }
                
                prompt.push_str("Ensure your response is valid JSON and follows this exact structure.");
                prompt
            }
        }
    }

    // Helper to convert JsonFieldType to string representation
    fn type_to_string(&self, field_type: &JsonFieldType) -> String {
        match field_type {
            JsonFieldType::String => "string".to_string(),
            JsonFieldType::Number => "number".to_string(),
            JsonFieldType::Boolean => "boolean".to_string(),
            JsonFieldType::Array(element_type) => format!("array of {}", self.type_to_string(element_type)),
            JsonFieldType::Object => "object".to_string(),
        }
    }
}
