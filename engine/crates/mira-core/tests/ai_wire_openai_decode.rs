#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format OpenAI Chat, décodage d'un événement SSE en deltas.

use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::openai_decode::decode_chat_event;
use mira_vocabulary::{AiError, ChatDelta};

/// Construit un événement SSE anonyme portant les données données.
fn event(data: &str) -> SseEvent {
    SseEvent {
        event: None,
        data: data.to_string(),
    }
}

/// Le texte vient de choices[0].delta.content
#[test]
fn decode_text_delta() {
    let data = r#"{"choices":[{"index":0,"delta":{"content":"Bon"},"finish_reason":null}]}"#;
    let deltas = decode_chat_event(&event(data)).unwrap();
    assert_eq!(deltas, vec![ChatDelta::Text { text: "Bon".into() }]);
}

/// Un morceau de rôle seul ou un contenu vide ne produit rien
#[test]
fn decode_ignores_empty_delta_and_role_only_chunk() {
    let role = r#"{"choices":[{"delta":{"role":"assistant","content":""}}]}"#;
    assert!(decode_chat_event(&event(role)).unwrap().is_empty());
    let empty = r#"{"choices":[{"delta":{}}]}"#;
    assert!(decode_chat_event(&event(empty)).unwrap().is_empty());
}

/// Le marqueur [DONE] ne produit aucun delta
#[test]
fn decode_done_marker_yields_nothing_new() {
    assert!(decode_chat_event(&event("[DONE]")).unwrap().is_empty());
}

/// Un JSON invalide est une réponse inattendue, sans copie du contenu reçu
#[test]
fn decode_invalid_json_is_bad_response() {
    let error = decode_chat_event(&event("secret {not json")).unwrap_err();
    assert_eq!(error, AiError::BadResponse);
    assert!(!format!("{error:?} {error}").contains("secret"));
}
