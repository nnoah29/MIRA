#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Messages, décodage d'un événement SSE : texte, maintien, erreurs.

use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::anthropic_decode::decode_chat_event;
use mira_vocabulary::{AiError, ChatDelta};

/// Construit un événement SSE anonyme portant les données données.
fn event(data: &str) -> SseEvent {
    SseEvent {
        event: None,
        data: data.to_string(),
    }
}

/// Le texte vient de delta.text d'un content_block_delta de type text_delta
#[test]
fn decode_content_block_delta_text() {
    let data =
        r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Bon"}}"#;
    let deltas = decode_chat_event(&event(data)).unwrap();
    assert_eq!(deltas, vec![ChatDelta::Text { text: "Bon".into() }]);
}

/// Un delta qui n'est pas du texte, ou un texte vide, ne produit rien
#[test]
fn decode_ignores_non_text_and_empty_deltas() {
    let other =
        r#"{"type":"content_block_delta","delta":{"type":"input_json_delta","partial_json":"{"}}"#;
    assert!(decode_chat_event(&event(other)).unwrap().is_empty());
    let empty = r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":""}}"#;
    assert!(decode_chat_event(&event(empty)).unwrap().is_empty());
}

/// Le signal de maintien et les événements de bloc ne produisent rien
#[test]
fn decode_ping_event_yields_nothing() {
    assert!(
        decode_chat_event(&event(r#"{"type":"ping"}"#))
            .unwrap()
            .is_empty()
    );
    let start =
        r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#;
    assert!(decode_chat_event(&event(start)).unwrap().is_empty());
    assert!(
        decode_chat_event(&event(r#"{"type":"content_block_stop","index":0}"#))
            .unwrap()
            .is_empty()
    );
}

/// Un événement d'erreur est un service indisponible, sans copie de son texte
#[test]
fn decode_error_event_is_unavailable() {
    let data = r#"{"type":"error","error":{"type":"overloaded_error","message":"secret"}}"#;
    let error = decode_chat_event(&event(data)).unwrap_err();
    assert_eq!(error, AiError::Unavailable);
    assert!(!format!("{error:?} {error}").contains("secret"));
}

/// Un JSON invalide est une réponse inattendue, sans copie du contenu reçu
#[test]
fn decode_invalid_json_is_bad_response() {
    let error = decode_chat_event(&event("secret {not json")).unwrap_err();
    assert_eq!(error, AiError::BadResponse);
    assert!(!format!("{error:?} {error}").contains("secret"));
}
