#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Responses, décodage d'un événement SSE en deltas.

use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::dispatch::decode_event;
use mira_vocabulary::{AiError, ChatDelta, StopReason, WireFormat};

/// Décode un événement portant les données données.
fn decode(data: &str) -> Result<Vec<ChatDelta>, AiError> {
    let event = SseEvent {
        event: None,
        data: data.to_string(),
    };
    decode_event(WireFormat::OpenAiResponses, &event)
}

/// Le texte vient de `response.output_text.delta`
#[test]
fn text_delta_is_decoded() {
    let data = r#"{"type":"response.output_text.delta","output_index":0,"delta":"Bon"}"#;
    assert_eq!(
        decode(data).unwrap(),
        vec![ChatDelta::Text { text: "Bon".into() }]
    );
    let empty = r#"{"type":"response.output_text.delta","delta":""}"#;
    assert!(decode(empty).unwrap().is_empty());
}

/// `response.completed` donne la consommation puis la fin de tour
#[test]
fn completed_gives_usage_then_end_turn() {
    let data = r#"{"type":"response.completed","response":{"output":[{"type":"message"}],"usage":{"input_tokens":12,"output_tokens":5}}}"#;
    let expected = vec![
        ChatDelta::Usage {
            input_tokens: 12,
            output_tokens: 5,
        },
        ChatDelta::Done {
            reason: StopReason::EndTurn,
        },
    ];
    assert_eq!(decode(data).unwrap(), expected);
}

/// Un appel de fonction dans la sortie termine le tour sur `ToolUse`
#[test]
fn completed_with_function_call_gives_tool_use() {
    let data = r#"{"type":"response.completed","response":{"output":[{"type":"function_call","call_id":"c1","name":"f"}],"usage":{"input_tokens":1,"output_tokens":2}}}"#;
    let deltas = decode(data).unwrap();
    assert_eq!(
        deltas.last(),
        Some(&ChatDelta::Done {
            reason: StopReason::ToolUse
        })
    );
}

/// Une réponse tronquée par la borne de sortie termine sur `MaxTokens`
#[test]
fn incomplete_max_tokens() {
    let data = r#"{"type":"response.incomplete","response":{"incomplete_details":{"reason":"max_output_tokens"},"usage":{"input_tokens":3,"output_tokens":4}}}"#;
    let deltas = decode(data).unwrap();
    assert_eq!(
        deltas.last(),
        Some(&ChatDelta::Done {
            reason: StopReason::MaxTokens
        })
    );
}
