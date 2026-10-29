#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Messages, décodage de la consommation et de la fin de message.

use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::anthropic_decode::decode_chat_event;
use mira_vocabulary::{ChatDelta, StopReason};

/// Construit un événement SSE anonyme portant les données données.
fn event(data: &str) -> SseEvent {
    SseEvent {
        event: None,
        data: data.to_string(),
    }
}

/// message_start porte la consommation d'entrée ; un champ absent vaut 0
#[test]
fn decode_message_start_gives_input_usage() {
    let data = r#"{"type":"message_start","message":{"id":"x","usage":{"input_tokens":25,"output_tokens":1}}}"#;
    let expected = ChatDelta::Usage {
        input_tokens: 25,
        output_tokens: 1,
    };
    assert_eq!(decode_chat_event(&event(data)).unwrap(), vec![expected]);
    let bare = r#"{"type":"message_start","message":{"usage":{"input_tokens":7}}}"#;
    let expected = ChatDelta::Usage {
        input_tokens: 7,
        output_tokens: 0,
    };
    assert_eq!(decode_chat_event(&event(bare)).unwrap(), vec![expected]);
}

/// message_delta porte la consommation de sortie puis la raison d'arrêt
#[test]
fn decode_message_delta_gives_output_usage_and_stop_reason() {
    let usage = ChatDelta::Usage {
        input_tokens: 0,
        output_tokens: 15,
    };
    let done = |reason| ChatDelta::Done { reason };
    let end = r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":15}}"#;
    let expected = vec![usage.clone(), done(StopReason::EndTurn)];
    assert_eq!(decode_chat_event(&event(end)).unwrap(), expected);
    let max = r#"{"type":"message_delta","delta":{"stop_reason":"max_tokens"},"usage":{"output_tokens":15}}"#;
    let expected = vec![usage, done(StopReason::MaxTokens)];
    assert_eq!(decode_chat_event(&event(max)).unwrap(), expected);
}

/// Un message_delta sans raison d'arrêt (null) ne produit pas de fin
#[test]
fn decode_message_delta_without_stop_reason_has_no_done() {
    let data =
        r#"{"type":"message_delta","delta":{"stop_reason":null},"usage":{"output_tokens":3}}"#;
    let deltas = decode_chat_event(&event(data)).unwrap();
    assert_eq!(deltas.len(), 1);
    assert!(matches!(
        deltas[0],
        ChatDelta::Usage {
            output_tokens: 3,
            ..
        }
    ));
}

/// message_stop signale la fin du flux
#[test]
fn decode_message_stop_gives_done() {
    let deltas = decode_chat_event(&event(r#"{"type":"message_stop"}"#)).unwrap();
    assert_eq!(
        deltas,
        vec![ChatDelta::Done {
            reason: StopReason::EndTurn
        }]
    );
}

/// Un événement sans données ou fait d'espaces ne produit rien
#[test]
fn decode_empty_or_blank_data_yields_nothing() {
    assert!(decode_chat_event(&event("")).unwrap().is_empty());
    assert!(decode_chat_event(&event("  \t ")).unwrap().is_empty());
}
