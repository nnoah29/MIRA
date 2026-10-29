//! Encodage d'une requête de discussion au format OpenAI Chat, sans en-tête d'authentification.

use mira_vocabulary::{AiProviderDescriptor, ChatMessage, ChatRequest, ChatRole, MaxTokensField};
use serde_json::{Map, Value, json};

use super::call::HttpCall;

pub(super) const CHAT_PATH: &str = "/chat/completions";
const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 4096;

/// Construit l'appel de discussion diffusée ; la clé est ajoutée plus tard par le transport.
pub fn encode_chat(
    request: &ChatRequest,
    model: &str,
    descriptor: &AiProviderDescriptor,
) -> HttpCall {
    let mut body = Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("messages".to_string(), Value::Array(wire_messages(request)));
    body.insert("stream".to_string(), json!(true));
    body.insert("stream_options".to_string(), json!({"include_usage": true}));
    let limit = request
        .max_output_tokens
        .unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS);
    body.insert(max_tokens_name(descriptor).to_string(), json!(limit));
    if let Some(temperature) = request.temperature {
        body.insert("temperature".to_string(), temperature_value(temperature));
    }
    HttpCall {
        path: CHAT_PATH.to_string(),
        headers: request_headers(descriptor),
        body: Value::Object(body).to_string(),
    }
}

/// Liste les messages du fil, la consigne en premier.
fn wire_messages(request: &ChatRequest) -> Vec<Value> {
    let system = request
        .system
        .iter()
        .map(|text| json!({"role": "system", "content": text}));
    let others = request.messages.iter().map(wire_message);
    system.chain(others).collect()
}

/// Traduit un message du fil en objet du format.
fn wire_message(message: &ChatMessage) -> Value {
    let role = match message.role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
    };
    json!({"role": role, "content": message.text})
}

/// Donne le nom du champ qui borne la réponse selon les particularités du service.
fn max_tokens_name(descriptor: &AiProviderDescriptor) -> &'static str {
    match descriptor.quirks.max_tokens_field {
        MaxTokensField::MaxTokens => "max_tokens",
        MaxTokensField::MaxCompletionTokens => "max_completion_tokens",
    }
}

/// Convertit la température sans bruit de précision (0.7 reste 0.7).
fn temperature_value(temperature: f32) -> Value {
    match temperature.to_string().parse::<f64>() {
        Ok(number) => json!(number),
        Err(_) => json!(temperature),
    }
}

/// Réunit les en-têtes de contenu et ceux, statiques, de la fiche du service.
fn request_headers(descriptor: &AiProviderDescriptor) -> Vec<(String, String)> {
    let mut headers = vec![
        ("content-type".to_string(), "application/json".to_string()),
        ("accept".to_string(), "text/event-stream".to_string()),
    ];
    headers.extend(descriptor.static_headers.iter().cloned());
    headers
}
