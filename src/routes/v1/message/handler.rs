use crate::adapter::services::AdapterRegistry;
use crate::adapter::traits::{AdapterService, LlmAdapter};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson, Response},
};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{
    request::{Message, MessageRequest},
    response::{MessageResponse, Usage},
};

/// Handler for sending messages to recipients via AI adapters
pub async fn send_message(
    Path(recipient_id): Path<String>,
    State(adapter_registry): State<Arc<RwLock<AdapterRegistry>>>,
    Json(request): Json<MessageRequest>,
) -> Result<Response, StatusCode> {
    tracing::info!("Sending message to recipient: {}", recipient_id);

    // Get the default LLM adapter
    let mut registry = adapter_registry.write().await;
    let adapter = match registry.get_default_llm_adapter() {
        Some(_) => {
            // We need mutable access to call send_message
            match registry.get_llm_adapter_mut("ollama") {
                Some(adapter) => adapter,
                None => {
                    tracing::error!("No Ollama LLM adapter available");
                    return create_error_response("No Ollama LLM adapter configured");
                }
            }
        }
        None => {
            tracing::error!("No LLM adapter available");
            return create_error_response("No LLM adapter configured");
        }
    };

    // For now, use the last message content as the prompt
    let prompt = match request.messages.last() {
        Some(msg) => &msg.content,
        None => {
            return create_error_response("No messages provided");
        }
    };

    // Send message to LLM adapter
    match adapter.send_message(prompt).await {
        Ok(content) => {
            let response_message = Message {
                role: "assistant".to_string(),
                content,
            };

            let response = MessageResponse {
                success: true,
                message: response_message,
                model: format!("{}_model", adapter.provider_name()),
                finish_reason: Some("stop".to_string()),
                usage: Some(Usage {
                    prompt_tokens: estimate_token_count(prompt),
                    completion_tokens: 0, // TODO: Get from adapter
                    total_tokens: 0,      // TODO: Calculate properly
                }),
                timestamp: Utc::now().to_rfc3339(),
            };

            Ok(ResponseJson(response).into_response())
        }
        Err(e) => {
            tracing::error!("LLM adapter error: {}", e);
            create_error_response(&format!("AI request failed: {}", e))
        }
    }
}

/// Create an error response
fn create_error_response(error_msg: &str) -> Result<Response, StatusCode> {
    let response_message = Message {
        role: "assistant".to_string(),
        content: error_msg.to_string(),
    };

    let response = MessageResponse {
        success: false,
        message: response_message,
        model: "error".to_string(),
        finish_reason: Some("error".to_string()),
        usage: None,
        timestamp: Utc::now().to_rfc3339(),
    };

    Ok(ResponseJson(response).into_response())
}

/// Simple token count estimation (rough approximation)
fn estimate_token_count(text: &str) -> u32 {
    // Very rough estimate: ~4 characters per token
    (text.len() / 4).max(1) as u32
}
