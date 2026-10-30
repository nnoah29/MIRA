#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Types de l'IA : forme sur le fil des deltas, valeur par défaut des outils, absence de clé dans une connexion.

use mira_vocabulary::{
    AiConnection, AiConnectionId, AiModel, AiModelList, AiProviderId, ChatDelta, ChatRequest,
    StopReason, Timestamp,
};
use serde_json::json;

/// Un delta porte son type en `snake_case`
#[test]
fn chat_delta_serializes_with_type_tag() {
    let text = ChatDelta::Text { text: "a".into() };
    assert_eq!(
        serde_json::to_value(&text).unwrap(),
        json!({"type": "text", "text": "a"})
    );
    let usage = ChatDelta::Usage {
        input_tokens: 3,
        output_tokens: 5,
    };
    let expected = json!({"type": "usage", "input_tokens": 3, "output_tokens": 5});
    assert_eq!(serde_json::to_value(&usage).unwrap(), expected);
    let end = ChatDelta::Done {
        reason: StopReason::EndTurn,
    };
    let expected = json!({"type": "done", "reason": "end_turn"});
    assert_eq!(serde_json::to_value(&end).unwrap(), expected);
}

/// Une fin en échec garde son code et le délai conseillé, et se relit à l'identique
#[test]
fn failed_done_delta_keeps_code_and_retry_delay() {
    let failed = ChatDelta::Done {
        reason: StopReason::Failed {
            code: "server.ai_rate_limited".into(),
            retry_after_secs: Some(7),
        },
    };
    let value = serde_json::to_value(&failed).unwrap();
    let expected = json!({
        "type": "done",
        "reason": {"failed": {"code": "server.ai_rate_limited", "retry_after_secs": 7}}
    });
    assert_eq!(value, expected);
    assert_eq!(serde_json::from_value::<ChatDelta>(value).unwrap(), failed);
}

/// Une requête sans champ `tools` se relit avec une liste d'outils vide
#[test]
fn chat_request_defaults_tools_to_empty() {
    let raw = json!({
        "model": null,
        "system": "Sois bref",
        "messages": [{"role": "user", "text": "Bonjour"}],
        "max_output_tokens": 256,
        "temperature": null
    });
    let request: ChatRequest = serde_json::from_value(raw).unwrap();
    assert!(request.tools.is_empty());
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.max_output_tokens, Some(256));
}

/// Connexion complète, avec clé présente, consentement et liste de modèles
fn sample_connection() -> AiConnection {
    AiConnection {
        id: AiConnectionId::from_raw("conn-1"),
        provider: AiProviderId("openai".into()),
        label: "Perso".into(),
        base_url: None,
        default_model: Some("m1".into()),
        is_default: true,
        has_key: true,
        consent_at: Some(Timestamp::from_millis(1)),
        models: Some(AiModelList {
            models: vec![AiModel {
                id: "m1".into(),
                created_at: None,
            }],
            fetched_at: Timestamp::from_millis(2),
        }),
    }
}

/// Une connexion n'expose jamais la clé : seul l'indicateur `has_key` existe
#[test]
fn connection_never_serializes_a_key_field() {
    let value = serde_json::to_value(sample_connection()).unwrap();
    let mut names: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    let expected = [
        "base_url",
        "consent_at",
        "default_model",
        "has_key",
        "id",
        "is_default",
        "label",
        "models",
        "provider",
    ];
    assert_eq!(names, expected);
    assert!(!value.to_string().to_lowercase().contains("secret"));
}
