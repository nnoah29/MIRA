#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Conversation de l'assistant IA : métadonnées aplaties avec le payload, et codes d'erreur de l'historique.

use mira_vocabulary::error::codes::{self, incompatible_state, invalid_input, not_found};
use mira_vocabulary::{AiConversation, AiConversationMeta, Timestamp};

/// Une conversation d'exemple
fn sample() -> AiConversation {
    AiConversation {
        meta: AiConversationMeta {
            id: "c-1".into(),
            title: "Mon titre".into(),
            created_at: Timestamp::from_millis(10),
            updated_at: Timestamp::from_millis(20),
            payload_version: 1,
        },
        payload: "{\"messages\":[]}".into(),
    }
}

/// La conversation s'écrit à plat et se relit à l'identique
#[test]
fn conversation_round_trips_and_flattens_meta() {
    let value = serde_json::to_value(sample()).unwrap();
    let object = value.as_object().unwrap();
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "created_at",
            "id",
            "payload",
            "payload_version",
            "title",
            "updated_at"
        ]
    );
    let back: AiConversation = serde_json::from_value(value).unwrap();
    assert_eq!(back, sample());
}

/// Les cinq codes de l'historique ont leur texte stable et figurent au catalogue
#[test]
fn history_codes_are_stable_and_registered() {
    let expected = [
        (
            invalid_input::AI_CONVERSATION_TITLE,
            "invalid_input.ai_conversation_title",
        ),
        (
            invalid_input::AI_CONVERSATION_TOO_LARGE,
            "invalid_input.ai_conversation_too_large",
        ),
        (
            invalid_input::AI_CONVERSATION_PAYLOAD,
            "invalid_input.ai_conversation_payload",
        ),
        (
            incompatible_state::AI_HISTORY_DISABLED,
            "incompatible_state.ai_history_disabled",
        ),
        (not_found::AI_CONVERSATION, "not_found.ai_conversation"),
    ];
    for (code, text) in expected {
        assert_eq!(code.0, text);
        assert!(codes::ALL.contains(&code), "{text} absent du catalogue");
    }
}
