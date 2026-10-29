#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format OpenAI Chat, décodage de la consommation et de la raison d'arrêt.

use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::openai_decode::decode_chat_event;
use mira_vocabulary::{ChatDelta, StopReason};

/// Construit un événement SSE anonyme portant les données données.
fn event(data: &str) -> SseEvent {
    SseEvent {
        event: None,
        data: data.to_string(),
    }
}

/// Le morceau final sans choices porte la consommation ; un champ absent vaut 0
#[test]
fn decode_usage_chunk() {
    let data = r#"{"choices":[],"usage":{"prompt_tokens":12,"completion_tokens":34}}"#;
    let expected = ChatDelta::Usage {
        input_tokens: 12,
        output_tokens: 34,
    };
    assert_eq!(decode_chat_event(&event(data)).unwrap(), vec![expected]);
    let partial = r#"{"usage":{"completion_tokens":5}}"#;
    let expected = ChatDelta::Usage {
        input_tokens: 0,
        output_tokens: 5,
    };
    assert_eq!(decode_chat_event(&event(partial)).unwrap(), vec![expected]);
}

/// stop donne EndTurn, length donne MaxTokens
#[test]
fn decode_finish_reason_stop_and_length() {
    let stop = r#"{"choices":[{"delta":{},"finish_reason":"stop"}]}"#;
    let length = r#"{"choices":[{"delta":{},"finish_reason":"length"}]}"#;
    let done = |reason| vec![ChatDelta::Done { reason }];
    assert_eq!(
        decode_chat_event(&event(stop)).unwrap(),
        done(StopReason::EndTurn)
    );
    assert_eq!(
        decode_chat_event(&event(length)).unwrap(),
        done(StopReason::MaxTokens)
    );
}
