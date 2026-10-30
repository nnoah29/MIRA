#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Messages, décodage des appels d'outils : début de bloc, morceaux d'arguments, raison d'arrêt.

use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::anthropic_decode::decode_chat_event;
use mira_vocabulary::{ChatDelta, StopReason};

/// Décode des données SSE anonymes.
fn decode(data: &str) -> Vec<ChatDelta> {
    let event = SseEvent {
        event: None,
        data: data.to_string(),
    };
    decode_chat_event(&event).unwrap()
}

/// Le début d'un bloc d'outil donne l'identifiant et le nom, sans argument
#[test]
fn start_block_yields_named_fragment() {
    let data = r#"{"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"toolu_1","name":"search_messages","input":{}}}"#;
    let expected = ChatDelta::ToolFragment {
        index: 1,
        id: Some("toolu_1".into()),
        name: Some("search_messages".into()),
        arguments: String::new(),
    };
    assert_eq!(decode(data), vec![expected]);
}

/// Un morceau d'arguments garde son rang de bloc
#[test]
fn json_delta_yields_argument_fragment() {
    let data = r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"query\":"}}"#;
    let expected = ChatDelta::ToolFragment {
        index: 1,
        id: None,
        name: None,
        arguments: "{\"query\":".into(),
    };
    assert_eq!(decode(data), vec![expected]);
}

/// L'arrêt pour outil se distingue de la fin normale
#[test]
fn tool_use_stop_reason_is_tool_use() {
    let data = r#"{"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":9}}"#;
    let last = decode(data).pop().unwrap();
    assert_eq!(
        last,
        ChatDelta::Done {
            reason: StopReason::ToolUse
        }
    );
}

/// Un bloc de texte qui s'ouvre ne produit rien
#[test]
fn text_block_start_is_ignored() {
    let data =
        r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#;
    assert!(decode(data).is_empty());
}

/// Un morceau d'arguments vide ne produit rien
#[test]
fn empty_partial_json_is_ignored() {
    let data = r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":""}}"#;
    assert!(decode(data).is_empty());
}
