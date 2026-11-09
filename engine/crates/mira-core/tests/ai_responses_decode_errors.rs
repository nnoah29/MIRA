#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Responses, décodage des échecs et des événements inconnus.

use mira_core::ai::sse::SseEvent;
use mira_core::ai::wire::dispatch::decode_event;
use mira_vocabulary::{AiError, ChatDelta, WireFormat};

/// Décode un événement portant les données données.
fn decode(data: &str) -> Result<Vec<ChatDelta>, AiError> {
    let event = SseEvent {
        event: None,
        data: data.to_string(),
    };
    decode_event(WireFormat::OpenAiResponses, &event)
}

/// Le plafond d'usage de l'abonnement est un quota épuisé
#[test]
fn failed_usage_limit_is_quota_exhausted() {
    let data = r#"{"type":"response.failed","response":{"error":{"code":"subscription_sharing_usage_limit_exceeded","message":"SECRET-TEXT"}}}"#;
    assert_eq!(decode(data).unwrap_err(), AiError::QuotaExhausted);
}

/// L'usage indisponible est un service indisponible
#[test]
fn failed_usage_unavailable_is_unavailable() {
    let data = r#"{"type":"response.failed","response":{"error":{"code":"subscription_sharing_usage_unavailable"}}}"#;
    assert_eq!(decode(data).unwrap_err(), AiError::Unavailable);
}

/// Un autre échec est une réponse inattendue, sans copie du message reçu
#[test]
fn failed_unknown_is_bad_response() {
    let data =
        r#"{"type":"response.failed","response":{"error":{"code":"x","message":"SECRET-TEXT"}}}"#;
    let error = decode(data).unwrap_err();
    assert_eq!(error, AiError::BadResponse);
    assert!(!format!("{error:?}{error}").contains("SECRET-TEXT"));
}

/// Un événement inconnu est ignoré, un JSON invalide est refusé
#[test]
fn unknown_event_is_ignored_and_invalid_json_is_bad_response() {
    assert!(
        decode(r#"{"type":"response.created","response":{}}"#)
            .unwrap()
            .is_empty()
    );
    assert!(decode("").unwrap().is_empty());
    assert_eq!(decode("pas du json").unwrap_err(), AiError::BadResponse);
}
