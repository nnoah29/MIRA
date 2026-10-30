#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format OpenAI Chat, encodage : consigne, diffusion, température.

mod ai_wire_support;

use ai_wire_support::{descriptor, request};
use mira_core::ai::wire::openai_encode::encode_chat;
use mira_vocabulary::{ChatMessage, ChatRole, MaxTokensField};
use serde_json::Value;

/// La consigne devient le premier message, de rôle system, suivie des messages dans l'ordre
#[test]
fn encode_puts_system_as_first_message() {
    let mut req = request();
    req.messages.push(ChatMessage {
        role: ChatRole::Assistant,
        text: "Salut".to_string(),
    });
    let call = encode_chat(&req, "m1", &descriptor(MaxTokensField::MaxTokens));
    let body: Value = serde_json::from_str(&call.body).unwrap();
    let expected = serde_json::json!([
        {"role": "system", "content": "Sois bref."},
        {"role": "user", "content": "Bonjour"},
        {"role": "assistant", "content": "Salut"}
    ]);
    assert_eq!(body["messages"], expected);
    assert_eq!(body["model"], "m1");
    assert_eq!(call.path, "/chat/completions");
}

/// La requête demande la diffusion et le relevé de consommation dans le dernier morceau
#[test]
fn encode_sets_stream_and_usage_option() {
    let call = encode_chat(&request(), "m1", &descriptor(MaxTokensField::MaxTokens));
    let body: Value = serde_json::from_str(&call.body).unwrap();
    assert_eq!(body["stream"], true);
    assert_eq!(
        body["stream_options"],
        serde_json::json!({"include_usage": true})
    );
}

/// Sans température demandée, le champ n'est pas envoyé ; avec une valeur, elle part telle quelle
#[test]
fn encode_omits_temperature_when_none() {
    let descriptor = descriptor(MaxTokensField::MaxTokens);
    let call = encode_chat(&request(), "m1", &descriptor);
    let body: Value = serde_json::from_str(&call.body).unwrap();
    assert!(body.get("temperature").is_none());
    let mut req = request();
    req.temperature = Some(0.7);
    let body: Value = serde_json::from_str(&encode_chat(&req, "m1", &descriptor).body).unwrap();
    assert_eq!(body["temperature"], 0.7);
}
