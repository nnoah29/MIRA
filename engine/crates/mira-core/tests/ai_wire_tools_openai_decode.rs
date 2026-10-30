#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format OpenAI Chat, décodage des appels d'outils : premier morceau, suite, deux appels, arrêt.

use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::openai_decode::decode_chat_event;
use mira_vocabulary::{ChatDelta, StopReason};

/// Décode des données SSE anonymes.
fn decode(data: &str) -> Vec<ChatDelta> {
    let event = SseEvent {
        event: None,
        data: data.to_string(),
    };
    decode_chat_event(&event).unwrap()
}

/// Construit le fragment attendu.
fn fragment(index: u32, id: Option<&str>, name: Option<&str>, arguments: &str) -> ChatDelta {
    ChatDelta::ToolFragment {
        index,
        id: id.map(str::to_string),
        name: name.map(str::to_string),
        arguments: arguments.to_string(),
    }
}

/// Le premier morceau d'un appel porte l'identifiant et le nom
#[test]
fn first_delta_carries_id_and_name() {
    let data = r#"{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"search_messages","arguments":""}}]}}]}"#;
    let expected = fragment(0, Some("call_1"), Some("search_messages"), "");
    assert_eq!(decode(data), vec![expected]);
}

/// Les morceaux suivants ne portent que des arguments
#[test]
fn later_delta_carries_only_arguments() {
    let data =
        r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"que"}}]}}]}"#;
    assert_eq!(decode(data), vec![fragment(0, None, None, "{\"que")]);
}

/// Deux appels dans un morceau gardent leurs rangs
#[test]
fn two_calls_keep_their_indexes() {
    let data = r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"a"}},{"index":1,"function":{"arguments":"b"}}]}}]}"#;
    let expected = vec![fragment(0, None, None, "a"), fragment(1, None, None, "b")];
    assert_eq!(decode(data), expected);
}

/// L'arrêt pour outils se distingue de la fin normale
#[test]
fn tool_calls_finish_is_tool_use() {
    let data = r#"{"choices":[{"delta":{},"finish_reason":"tool_calls"}]}"#;
    let done = ChatDelta::Done {
        reason: StopReason::ToolUse,
    };
    assert_eq!(decode(data), vec![done]);
}
