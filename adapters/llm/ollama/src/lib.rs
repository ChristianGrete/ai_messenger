use wit_bindgen::generate;

generate!({
    world: "llm-adapter",
    path: "wit"
});

use exports::ai_messenger::llm::llm::{
    ChatRequest, ChatResponse, HttpConfig, HttpResponse, StreamChunk,
};

use ai_messenger::llm::types::{FinishReason, Role, Usage};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

struct OllamaAdapter;

impl exports::ai_messenger::llm::llm::Guest for OllamaAdapter {
    fn prepare_request(request: ChatRequest) -> Result<HttpConfig, String> {
        // Convert generic chat request to Ollama API format
        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|msg| OllamaMessage {
                    role: role_to_string(msg.role.clone()),
                    content: msg.content.clone(),
                })
                .collect(),
            options: build_ollama_options(&request),
            stream: request.enable_streaming.unwrap_or(false),
        };

        let body = serde_json::to_string(&ollama_request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        // Parse provider-specific config if provided
        let base_url = if let Some(params) = &request.provider_params {
            let config: HashMap<String, serde_json::Value> =
                serde_json::from_str(params).unwrap_or_default();
            config
                .get("base_url")
                .and_then(|v| v.as_str())
                .unwrap_or("http://localhost:11434")
                .to_string()
        } else {
            "http://localhost:11434".to_string()
        };

        Ok(HttpConfig {
            url: format!("{}/api/chat", base_url),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Accept".to_string(), "application/json".to_string()),
            ],
            body,
        })
    }

    fn parse_response(response: HttpResponse) -> Result<ChatResponse, String> {
        if response.status_code != 200 {
            return Err(format!("HTTP error: {}", response.status_code));
        }

        let ollama_response: OllamaChatResponse = serde_json::from_str(&response.body)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(ChatResponse {
            content: ollama_response.message.content,
            model: ollama_response.model,
            finish_reason: Some(FinishReason::Stop), // Ollama doesn't provide detailed finish reasons
            usage: ollama_response
                .prompt_eval_count
                .map(|prompt_tokens| Usage {
                    prompt_tokens,
                    completion_tokens: ollama_response.eval_count.unwrap_or(0),
                    total_tokens: prompt_tokens + ollama_response.eval_count.unwrap_or(0),
                }),
        })
    }

    fn parse_stream_chunk(chunk: String) -> Result<Option<StreamChunk>, String> {
        if chunk.trim().is_empty() {
            return Ok(None);
        }

        // Ollama streams JSON objects separated by newlines
        let ollama_chunk: OllamaStreamChunk = serde_json::from_str(&chunk)
            .map_err(|e| format!("Failed to parse stream chunk: {}", e))?;

        Ok(Some(StreamChunk {
            sequence: 0, // Ollama doesn't provide sequence numbers
            content: ollama_chunk.message.content,
            is_final: ollama_chunk.done,
            usage: if ollama_chunk.done {
                ollama_chunk.prompt_eval_count.map(|prompt_tokens| Usage {
                    prompt_tokens,
                    completion_tokens: ollama_chunk.eval_count.unwrap_or(0),
                    total_tokens: prompt_tokens + ollama_chunk.eval_count.unwrap_or(0),
                })
            } else {
                None
            },
            finish_reason: if ollama_chunk.done {
                Some(FinishReason::Stop)
            } else {
                None
            },
        }))
    }
}

// Ollama API types
#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    options: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaMessage,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaStreamChunk {
    #[allow(dead_code)]
    model: String,
    message: OllamaMessage,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

// Helper functions
fn role_to_string(role: Role) -> String {
    match role {
        Role::System => "system".to_string(),
        Role::User => "user".to_string(),
        Role::Assistant => "assistant".to_string(),
        Role::Function => "function".to_string(),
        Role::Tool => "tool".to_string(),
        Role::Other(s) => s,
    }
}

fn build_ollama_options(request: &ChatRequest) -> HashMap<String, serde_json::Value> {
    let mut options = HashMap::new();

    if let Some(temp) = request.temperature {
        options.insert("temperature".to_string(), serde_json::Value::from(temp));
    }

    if let Some(top_p) = request.top_p {
        options.insert("top_p".to_string(), serde_json::Value::from(top_p));
    }

    if let Some(max_tokens) = request.max_completion_tokens {
        options.insert(
            "num_predict".to_string(),
            serde_json::Value::from(max_tokens),
        );
    }

    if let Some(stop) = &request.stop {
        if !stop.is_empty() {
            options.insert("stop".to_string(), serde_json::Value::from(stop.clone()));
        }
    }

    if let Some(seed) = request.seed {
        options.insert("seed".to_string(), serde_json::Value::from(seed));
    }

    options
}

export!(OllamaAdapter);
