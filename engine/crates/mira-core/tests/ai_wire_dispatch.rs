#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Aiguillage par format : chemin, requête, événement et liste de modèles.

mod ai_wire_support;

use ai_wire_support::{descriptor, request};
use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::dispatch::{chat_path, decode_event, decode_models, encode};
use mira_vocabulary::{ChatDelta, MaxTokensField, StopReason, WireFormat};

/// Construit un événement SSE anonyme portant les données données.
fn event(data: &str) -> SseEvent {
    SseEvent {
        event: None,
        data: data.to_string(),
    }
}

/// Chaque format a son chemin de discussion
#[test]
fn dispatch_chat_path_per_format() {
    assert_eq!(chat_path(WireFormat::OpenAiChat), "/chat/completions");
    assert_eq!(chat_path(WireFormat::AnthropicMessages), "/messages");
}

/// Un événement de chaque format passe par le bon décodeur
#[test]
fn dispatch_routes_each_format() {
    let openai = r#"{"choices":[{"delta":{"content":"A"}}]}"#;
    let messages = r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"B"}}"#;
    let text = |t: &str| vec![ChatDelta::Text { text: t.into() }];
    assert_eq!(
        decode_event(WireFormat::OpenAiChat, &event(openai)).unwrap(),
        text("A")
    );
    assert_eq!(
        decode_event(WireFormat::AnthropicMessages, &event(messages)).unwrap(),
        text("B")
    );
    let stop = decode_event(
        WireFormat::AnthropicMessages,
        &event(r#"{"type":"message_stop"}"#),
    );
    assert_eq!(
        stop.unwrap(),
        vec![ChatDelta::Done {
            reason: StopReason::EndTurn
        }]
    );
}

/// La requête est encodée selon le format du descripteur
#[test]
fn dispatch_encodes_by_descriptor_format() {
    let mut descriptor = descriptor(MaxTokensField::MaxTokens);
    let openai = encode(&request(), "m1", &descriptor);
    assert_eq!(openai.path, chat_path(WireFormat::OpenAiChat));
    descriptor.format = WireFormat::AnthropicMessages;
    let messages = encode(&request(), "m1", &descriptor);
    assert_eq!(messages.path, chat_path(WireFormat::AnthropicMessages));
    assert!(messages.body.contains(r#""system":"Sois bref.""#));
}

/// La liste des modèles est décodée selon le format, avec la date propre à chacun
#[test]
fn dispatch_decodes_models_by_format() {
    let openai = br#"{"data":[{"id":"a","created":5}]}"#;
    let messages = br#"{"data":[{"id":"b","created_at":"1970-01-01T00:00:07Z"}]}"#;
    let first = decode_models(WireFormat::OpenAiChat, openai).unwrap();
    assert_eq!((first[0].id.as_str(), first[0].created_at), ("a", Some(5)));
    let second = decode_models(WireFormat::AnthropicMessages, messages).unwrap();
    assert_eq!(
        (second[0].id.as_str(), second[0].created_at),
        ("b", Some(7))
    );
}
