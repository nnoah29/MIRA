#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Erreurs, codes et événements de l'IA : table de correspondance, catalogue, message sans contenu libre.

use std::collections::HashSet;

use mira_vocabulary::error::codes::{
    self, authentication, cancelled, incompatible_state, invalid_input, not_found, server,
};
use mira_vocabulary::{AiError, ErrorCode, EventCategory, EventKind, MiraError, OperationId};

/// Table attendue : chaque variante d'`AiError` et son code stable.
fn table() -> [(AiError, ErrorCode); 9] {
    [
        (AiError::KeyRejected, authentication::AI_KEY_REJECTED),
        (AiError::ModelNotFound, not_found::AI_MODEL),
        (
            AiError::RateLimited {
                retry_after_secs: Some(7),
            },
            server::AI_RATE_LIMITED,
        ),
        (AiError::InputTooLarge, invalid_input::AI_INPUT_TOO_LARGE),
        (AiError::UrlInvalid, invalid_input::AI_URL_INVALID),
        (AiError::Unavailable, server::AI_UNAVAILABLE),
        (AiError::BadResponse, server::AI_BAD_RESPONSE),
        (
            AiError::ConsentRequired,
            incompatible_state::AI_CONSENT_REQUIRED,
        ),
        (AiError::Cancelled, cancelled::BY_USER),
    ]
}

/// Chaque variante d'`AiError` donne son code stable, et `MiraError` le reprend
#[test]
fn every_ai_error_maps_to_its_code() {
    for (error, code) in table() {
        assert_eq!(error.code(), code);
        let mira: MiraError = error.into();
        assert_eq!(mira.code, code.0);
        assert!(
            mira.code.starts_with(mira.category.as_str()),
            "{}",
            mira.code
        );
    }
}

/// Les treize nouveaux codes sont dans le catalogue, une seule fois, avec le préfixe de leur catégorie
#[test]
fn ai_codes_are_registered_in_all() {
    let new_codes: [ErrorCode; 13] = [
        authentication::AI_KEY_REJECTED,
        not_found::AI_MODEL,
        not_found::AI_CONNECTION,
        not_found::AI_PROVIDER,
        invalid_input::AI_INPUT_TOO_LARGE,
        invalid_input::AI_URL_INVALID,
        invalid_input::AI_KEY_INVALID,
        incompatible_state::AI_CONSENT_REQUIRED,
        incompatible_state::AI_KEY_MISSING,
        incompatible_state::AI_MODEL_MISSING,
        server::AI_RATE_LIMITED,
        server::AI_UNAVAILABLE,
        server::AI_BAD_RESPONSE,
    ];
    for code in &new_codes {
        let count = codes::ALL.iter().filter(|c| *c == code).count();
        assert_eq!(count, 1, "{}", code.0);
        assert!(code.0.contains(".ai_"), "{}", code.0);
    }
    let unique: HashSet<&str> = codes::ALL.iter().map(|c| c.0).collect();
    assert_eq!(unique.len(), codes::ALL.len());
}

/// Le message d'une erreur d'IA ne porte aucun contenu libre, pas même le délai conseillé
#[test]
fn ai_error_message_has_no_payload() {
    let error = AiError::RateLimited {
        retry_after_secs: Some(7),
    };
    let mira: MiraError = error.into();
    assert!(!mira.message.contains('7'));
    assert!(!mira.message.is_empty());
    assert!(mira.retryable);
    let rejected: MiraError = AiError::KeyRejected.into();
    assert!(!rejected.retryable);
}

/// Les deux événements de discussion appartiennent à la catégorie `ai` et ne désignent aucun compte
#[test]
fn ai_chat_events_have_ai_category() {
    let started = EventKind::AiChatStarted {
        operation: OperationId::from_raw("op-1"),
        provider: "openai".into(),
        model: "m1".into(),
    };
    let finished = EventKind::AiChatFinished {
        operation: OperationId::from_raw("op-1"),
        input_tokens: Some(3),
        output_tokens: None,
        duration_ms: 120,
        outcome: "end_turn".into(),
    };
    for kind in [&started, &finished] {
        assert_eq!(kind.category(), EventCategory::Ai);
        assert_eq!(kind.account(), None);
    }
    assert_eq!(serde_json::to_value(EventCategory::Ai).unwrap(), "ai");
    let value = serde_json::to_value(&started).unwrap();
    assert_eq!(value["type"], "ai_chat_started");
}
