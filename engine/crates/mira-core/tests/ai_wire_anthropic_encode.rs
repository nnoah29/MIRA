#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Messages, encodage : consigne de niveau supérieur, borne obligatoire, en-têtes.

mod ai_wire_support;

use ai_wire_support::{descriptor, request};
use mira_core::ai::wire::anthropic_encode::encode_chat;
use mira_vocabulary::{AiProviderDescriptor, MaxTokensField, WireFormat};
use serde_json::Value;

/// Descripteur générique parlant le protocole Messages, avec un en-tête de version statique.
fn messages_descriptor() -> AiProviderDescriptor {
    let mut descriptor = descriptor(MaxTokensField::MaxTokens);
    descriptor.format = WireFormat::AnthropicMessages;
    descriptor.static_headers = vec![("x-test-version".to_string(), "2023-06-01".to_string())];
    descriptor
}

/// La consigne est un champ de niveau supérieur ; les messages ne contiennent que le dialogue
#[test]
fn encode_system_is_top_level() {
    let call = encode_chat(&request(), "m1", &messages_descriptor());
    let body: Value = serde_json::from_str(&call.body).unwrap();
    assert_eq!(body["system"], "Sois bref.");
    let expected = serde_json::json!([{"role": "user", "content": "Bonjour"}]);
    assert_eq!(body["messages"], expected);
    assert_eq!(body["model"], "m1");
    assert_eq!(call.path, "/messages");
}

/// La borne de sortie est toujours envoyée : 4096 par défaut, la valeur de la requête sinon
#[test]
fn encode_always_sets_max_tokens() {
    let descriptor = messages_descriptor();
    let body: Value =
        serde_json::from_str(&encode_chat(&request(), "m1", &descriptor).body).unwrap();
    assert_eq!(body["max_tokens"], 4096);
    let mut req = request();
    req.max_output_tokens = Some(256);
    let body: Value = serde_json::from_str(&encode_chat(&req, "m1", &descriptor).body).unwrap();
    assert_eq!(body["max_tokens"], 256);
}

/// Les en-têtes statiques de la fiche sont recopiés, avec le type de contenu et l'accept SSE
#[test]
fn encode_copies_static_version_header() {
    let call = encode_chat(&request(), "m1", &messages_descriptor());
    let has = |name: &str, value: &str| call.headers.iter().any(|(n, v)| n == name && v == value);
    assert!(has("x-test-version", "2023-06-01"));
    assert!(has("content-type", "application/json"));
    assert!(has("accept", "text/event-stream"));
}

/// La requête demande la diffusion ; sans consigne ni température, ces champs sont absents
#[test]
fn encode_sets_stream_true() {
    let mut req = request();
    req.system = None;
    let call = encode_chat(&req, "m1", &messages_descriptor());
    let body: Value = serde_json::from_str(&call.body).unwrap();
    assert_eq!(body["stream"], true);
    assert!(body.get("system").is_none());
    assert!(body.get("temperature").is_none());
}

/// Aucun en-tête d'authentification : la clé est ajoutée plus tard par le transport
#[test]
fn encode_has_no_key_header() {
    let call = encode_chat(&request(), "m1", &messages_descriptor());
    for (name, _) in &call.headers {
        let lower = name.to_lowercase();
        assert!(!lower.contains("auth") && !lower.contains("key"), "{name}");
    }
    assert!(!call.body.contains("api_key"));
}
