use serde::{Deserialize, Serialize};

/// Message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Request body for sending messages
#[derive(Debug, Deserialize)]
pub struct MessageRequest {
    /// Optional sender ID - falls back to default if not provided
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)] // TODO: implement sender logic
    pub sender: Option<String>,

    /// Optional group ID - falls back to default if not provided
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)] // TODO: implement group logic
    pub group: Option<String>,

    /// Array of messages in the conversation
    #[allow(dead_code)] // TODO: implement message processing
    pub messages: Vec<Message>,

    /// Whether to stream the response (default: false)
    #[serde(default)]
    #[allow(dead_code)] // TODO: implement streaming
    pub stream: bool,
}
