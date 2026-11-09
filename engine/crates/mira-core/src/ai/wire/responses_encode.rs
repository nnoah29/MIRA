//! Encodage d'une requête de discussion au format Responses, sans en-tête d'authentification ni champ interdit.

use mira_vocabulary::{AiProviderDescriptor, ChatRequest};
use serde_json::{Map, Value, json};

use super::call::HttpCall;
use super::responses_tools::{message_items, tools_value};

pub(super) const CHAT_PATH: &str = "/responses";

/// Construit l'appel de discussion diffusée ; ni température ni borne de sortie, le service les refuse.
pub fn encode_chat(
    request: &ChatRequest,
    model: &str,
    descriptor: &AiProviderDescriptor,
) -> HttpCall {
    let mut body = Map::new();
    body.insert("model".to_string(), json!(model));
    body.insert("instructions".to_string(), json!(instructions(request)));
    body.insert("input".to_string(), Value::Array(input_items(request)));
    if let Some(tools) = tools_value(&request.tools) {
        body.insert("tools".to_string(), tools);
    }
    body.insert("store".to_string(), json!(false));
    body.insert("stream".to_string(), json!(true));
    HttpCall {
        path: CHAT_PATH.to_string(),
        headers: request_headers(descriptor),
        body: Value::Object(body).to_string(),
    }
}

/// Consigne système, vide quand la requête n'en a pas.
fn instructions(request: &ChatRequest) -> &str {
    request.system.as_deref().unwrap_or_default()
}

/// Éléments d'entrée du fil, dans l'ordre.
fn input_items(request: &ChatRequest) -> Vec<Value> {
    request.messages.iter().flat_map(message_items).collect()
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
