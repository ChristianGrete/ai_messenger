use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson, Response},
};
use chrono::Utc;

use super::{
    request::{Message, MessageRequest},
    response::{MessageResponse, Usage},
};

/// Placeholder handler for sending messages to recipients
pub async fn send_message(
    Path(_recipient_id): Path<String>,
    Json(_request): Json<MessageRequest>,
) -> Result<Response, StatusCode> {
    // Create a placeholder response message
    let response_message = Message {
        role: "assistant".to_string(),
        content: "This is a placeholder response. The message handler is not yet implemented."
            .to_string(),
    };

    let response = MessageResponse {
        success: true,
        message: response_message,
        model: "placeholder-model".to_string(),
        finish_reason: Some("stop".to_string()),
        usage: Some(Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }),
        timestamp: Utc::now().to_rfc3339(),
    };

    // Return JSON response
    Ok(ResponseJson(response).into_response())
}
