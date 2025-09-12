use super::request::Message;
use serde::{Deserialize, Serialize};

/// Successful message response
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub success: bool,
    pub message: Message,
    pub model: String,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
    pub timestamp: String,
}

/// Error response for message endpoint
#[derive(Debug, Serialize)]
#[allow(dead_code)] // TODO: implement proper HTTP error responses
pub struct MessageErrorResponse {
    pub success: bool,
    pub error: String,
    pub error_type: String,
    pub timestamp: String,
}

/// Usage statistics from AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
