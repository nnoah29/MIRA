#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Responses, encodage : champs obligatoires, champs interdits, chemin.

mod ai_wire_support;

use ai_wire_support::{descriptor, request};
use mira_core::ai::wire::dispatch::{chat_path, encode};
use mira_vocabulary::{AiProviderDescriptor, MaxTokensField, WireFormat};
use serde_json::Value;

const FORBIDDEN: [&str; 16] = [
    "background",
    "conversation",
    "max_output_tokens",
    "max_tool_calls",
    "metadata",
    "moderation",
    "multi_agent",
    "prompt",
    "prompt_cache_retention",
    "safety_identifier",
    "temperature",
    "top_logprobs",
    "top_p",
    "truncation",
    "user",
    "previous_response_id",
];

/// Fiche d'un service parlant le format Responses.
fn responses_descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        format: WireFormat::OpenAiResponses,
        ..descriptor(MaxTokensField::MaxTokens)
    }
}

/// Le corps porte le modèle, la consigne en `instructions`, `store:false` et `stream:true`
#[test]
fn responses_body_has_required_fields() {
    let call = encode(&request(), "m1", &responses_descriptor());
    let body: Value = serde_json::from_str(&call.body).unwrap();
    assert_eq!(body["model"], "m1");
    assert_eq!(body["instructions"], "Sois bref.");
    assert_eq!(body["store"], false);
    assert_eq!(body["stream"], true);
    let user =
        serde_json::json!({"role": "user", "content": [{"type": "input_text", "text": "Bonjour"}]});
    assert_eq!(body["input"][0], user);
    assert!(body.get("messages").is_none());
}

/// Aucun champ interdit n'est émis, même si la requête fixe une température et une borne de sortie
#[test]
fn responses_body_never_carries_forbidden_fields() {
    let mut req = request();
    req.temperature = Some(0.7);
    req.max_output_tokens = Some(100);
    let call = encode(&req, "m1", &responses_descriptor());
    let body: Value = serde_json::from_str(&call.body).unwrap();
    for name in FORBIDDEN {
        assert!(body.get(name).is_none(), "champ interdit : {name}");
    }
}

/// Le chemin de discussion est `/responses` et les en-têtes demandent le flux
#[test]
fn responses_path_and_headers() {
    let call = encode(&request(), "m1", &responses_descriptor());
    assert_eq!(call.path, "/responses");
    assert_eq!(chat_path(WireFormat::OpenAiResponses), "/responses");
    let accept = call.headers.iter().find(|(name, _)| name == "accept");
    assert_eq!(
        accept.map(|(_, value)| value.as_str()),
        Some("text/event-stream")
    );
}

/// Sans consigne, `instructions` reste présent, vide
#[test]
fn responses_without_system_sends_empty_instructions() {
    let mut req = request();
    req.system = None;
    let call = encode(&req, "m1", &responses_descriptor());
    let body: Value = serde_json::from_str(&call.body).unwrap();
    assert_eq!(body["instructions"], "");
}
