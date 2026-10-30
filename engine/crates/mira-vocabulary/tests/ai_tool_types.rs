#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Types des outils d'IA : compatibilité du message 1.2, forme sur le fil des deltas d'outils.

use mira_vocabulary::{ChatDelta, ChatMessage, ChatRole, ContentKind, StopReason, ToolState};
use serde_json::json;

/// Un message écrit pour le contrat 1.2 reste lisible
#[test]
fn chat_message_1_2_still_deserializes() {
    let message: ChatMessage =
        serde_json::from_value(json!({"role": "user", "text": "x"})).unwrap();
    assert_eq!(message, ChatMessage::plain(ChatRole::User, "x".to_string()));
    assert!(message.tool_calls.is_empty());
    assert!(message.tool_result.is_none());
}

/// La fin d'un outil porte son état en snake_case et le type refusé
#[test]
fn tool_finished_serializes_with_snake_case_state() {
    let finished = ChatDelta::ToolFinished {
        id: "c1".into(),
        state: ToolState::Denied,
        results: 0,
        sources: vec![],
        denied_kind: Some(ContentKind::MailList),
    };
    let expected = json!({
        "type": "tool_finished", "id": "c1", "state": "denied",
        "results": 0, "sources": [], "denied_kind": "mail_list"
    });
    assert_eq!(serde_json::to_value(&finished).unwrap(), expected);
}

/// L'annonce d'un appel porte le nom de l'outil et l'extrait d'argument
#[test]
fn tool_called_serializes_with_summary() {
    let called = ChatDelta::ToolCalled {
        id: "c1".into(),
        name: "search_messages".into(),
        summary: "devis mars".into(),
    };
    let expected = json!({
        "type": "tool_called", "id": "c1", "name": "search_messages", "summary": "devis mars"
    });
    assert_eq!(serde_json::to_value(&called).unwrap(), expected);
}

/// Un fragment d'appel fait l'aller-retour sur le fil
#[test]
fn tool_fragment_roundtrips() {
    let fragment = ChatDelta::ToolFragment {
        index: 1,
        id: Some("c1".into()),
        name: None,
        arguments: "{\"q\"".into(),
    };
    let text = serde_json::to_string(&fragment).unwrap();
    assert_eq!(serde_json::from_str::<ChatDelta>(&text).unwrap(), fragment);
}

/// Le plafond de tours a sa raison d'arrêt stable
#[test]
fn stop_reason_max_tool_rounds_serializes() {
    let end = ChatDelta::Done {
        reason: StopReason::MaxToolRounds,
    };
    let expected = json!({"type": "done", "reason": "max_tool_rounds"});
    assert_eq!(serde_json::to_value(&end).unwrap(), expected);
}
