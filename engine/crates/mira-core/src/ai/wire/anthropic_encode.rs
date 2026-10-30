//! Encodage d'une requête de discussion au format Messages, sans en-tête d'authentification.

use mira_vocabulary::{AiProviderDescriptor, ChatRequest};
use serde_json::{Map, Value, json};

use super::anthropic_tools::{message_values, tools_value};
use super::call::HttpCall;

pub(super) const CHAT_PATH: &str = "/messages";
const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 4096;

/// Construit l'appel de discussion diffusée ; la clé est ajoutée plus tard par le transport.
pub fn encode_chat(
    request: &ChatRequest,
    model: &str,
    descriptor: &AiProviderDescriptor,
) -> HttpCall {
    let mut body = Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("stream".to_string(), json!(true));
    let limit = request
        .max_output_tokens
        .unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS);
    body.insert("max_tokens".to_string(), json!(limit));
    insert_dialogue(&mut body, request);
    if let Some(temperature) = request.temperature {
        body.insert("temperature".to_string(), temperature_value(temperature));
    }
    HttpCall {
        path: CHAT_PATH.to_string(),
        headers: request_headers(descriptor),
        body: Value::Object(body).to_string(),
    }
}

/// Ajoute le fil de messages, les outils annoncés et la consigne de niveau supérieur.
fn insert_dialogue(body: &mut Map<String, Value>, request: &ChatRequest) {
    let messages = Value::Array(message_values(&request.messages));
    body.insert("messages".to_string(), messages);
    if let Some(tools) = tools_value(&request.tools) {
        body.insert("tools".to_string(), tools);
    }
    if let Some(system) = &request.system {
        body.insert("system".to_string(), json!(system));
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
