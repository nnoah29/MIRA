#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format OpenAI Chat, encodage : borne de sortie, en-têtes, outils ignorés.

mod ai_wire_support;

use ai_wire_support::{descriptor, request};
use mira_core::ai::wire::openai_encode::encode_chat;
use mira_vocabulary::{MaxTokensField, ToolSpec};
use serde_json::Value;

/// Le nom du champ de borne suit le descripteur ; la valeur par défaut est 4096
#[test]
fn encode_uses_max_tokens_field_from_quirks() {
    let call = encode_chat(&request(), "m1", &descriptor(MaxTokensField::MaxTokens));
    let body: Value = serde_json::from_str(&call.body).unwrap();
    assert_eq!(body["max_tokens"], 4096);
    assert!(body.get("max_completion_tokens").is_none());
    let mut req = request();
    req.max_output_tokens = Some(256);
    let other = descriptor(MaxTokensField::MaxCompletionTokens);
    let body: Value = serde_json::from_str(&encode_chat(&req, "m1", &other).body).unwrap();
    assert_eq!(body["max_completion_tokens"], 256);
    assert!(body.get("max_tokens").is_none());
}

/// Aucun en-tête d'authentification : la clé est ajoutée plus tard par le transport
#[test]
fn encode_never_contains_a_key_header() {
    let call = encode_chat(&request(), "m1", &descriptor(MaxTokensField::MaxTokens));
    for (name, _) in &call.headers {
        let lower = name.to_lowercase();
        assert!(!lower.contains("auth") && !lower.contains("key"), "{name}");
    }
    assert!(call.headers.iter().any(|(n, v)| n == "x-test" && v == "1"));
    assert!(!call.body.contains("api_key"));
}

/// Les outils de la requête ne sont pas lus dans ce sous-projet
#[test]
fn encode_ignores_tools() {
    let mut req = request();
    req.tools.push(ToolSpec {
        name: "search".to_string(),
        description: "d".to_string(),
        parameters_schema: "{}".to_string(),
    });
    let call = encode_chat(&req, "m1", &descriptor(MaxTokensField::MaxTokens));
    let body: Value = serde_json::from_str(&call.body).unwrap();
    assert!(body.get("tools").is_none());
    assert!(!call.body.contains("search"));
}
