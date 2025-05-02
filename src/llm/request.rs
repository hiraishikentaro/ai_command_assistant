use serde::{Deserialize, Serialize};

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Message to send to LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

impl Message {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// Response format specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

impl Default for ResponseFormat {
    fn default() -> Self {
        Self {
            format_type: "json_object".to_string(),
        }
    }
}

/// OpenAI Chat API Request
#[derive(Debug, Clone, Serialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: usize,
    pub response_format: ResponseFormat,
}

impl OpenAiRequest {
    /// Create a new OpenAI request with default values
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: Vec::new(),
            temperature: 0.2,
            top_p: 1.0,
            max_tokens: 500,
            response_format: ResponseFormat::default(),
        }
    }

    /// Add a message to the request
    pub fn add_message(&mut self, message: Message) -> &mut Self {
        self.messages.push(message);
        self
    }

    /// Set the system message
    pub fn with_system_message(&mut self, content: impl Into<String>) -> &mut Self {
        self.messages.insert(0, Message::system(content));
        self
    }

    /// Add a user message
    pub fn with_user_message(&mut self, content: impl Into<String>) -> &mut Self {
        self.messages.push(Message::user(content));
        self
    }

    /// Set the temperature
    pub fn with_temperature(&mut self, temperature: f32) -> &mut Self {
        self.temperature = temperature;
        self
    }

    /// Set the max tokens
    pub fn with_max_tokens(&mut self, max_tokens: usize) -> &mut Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the response format
    pub fn with_response_format(&mut self, format_type: impl Into<String>) -> &mut Self {
        self.response_format = ResponseFormat {
            format_type: format_type.into(),
        };
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let system_msg = Message::system("You are a helpful assistant");
        assert!(matches!(system_msg.role, MessageRole::System));
        assert_eq!(system_msg.content, "You are a helpful assistant");

        let user_msg = Message::user("What's the weather like today?");
        assert!(matches!(user_msg.role, MessageRole::User));
        assert_eq!(user_msg.content, "What's the weather like today?");
    }

    #[test]
    fn test_request_builder() {
        let mut request = OpenAiRequest::new("gpt-4");
        request
            .with_system_message("You are a helpful assistant")
            .with_user_message("What's the weather like today?")
            .with_temperature(0.7)
            .with_max_tokens(1000);

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.temperature, 0.7);
        assert_eq!(request.max_tokens, 1000);
        assert_eq!(request.response_format.format_type, "json_object");
    }
}
