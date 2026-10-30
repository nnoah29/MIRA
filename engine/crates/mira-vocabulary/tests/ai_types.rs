#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Types de l'IA : forme sur le fil des deltas, valeur par défaut des outils, absence de clé dans une connexion.

use mira_vocabulary::{
    AiConnection, AiConnectionId, AiModel, AiModelList, AiProviderId, ChatDelta, ChatRequest,
    ContentKind, MaskingSettings, SensitiveCategory, StopReason, Timestamp,
};
use serde_json::{Value, json};

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
        allowed_kinds: vec![ContentKind::MailMessage],
        masking: MaskingSettings::remote_default(),
    }
}

/// Noms de toutes les clés d'objet du JSON, à tous les niveaux
fn key_names(value: &Value, names: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (name, inner) in map {
                names.push(name.clone());
                key_names(inner, names);
            }
        }
        Value::Array(items) => items.iter().for_each(|inner| key_names(inner, names)),
        _ => {}
    }
}

const CONNECTION_KEYS: [&str; 11] = [
    "allowed_kinds",
    "base_url",
    "consent_at",
    "default_model",
    "has_key",
    "id",
    "is_default",
    "label",
    "masking",
    "models",
    "provider",
];
const SENSITIVE_WORDS: [&str; 6] = [
    "key",
    "secret",
    "token",
    "password",
    "bearer",
    "authorization",
];

/// Une connexion n'expose jamais la clé : seul l'indicateur `has_key` existe, et aucun nom de champ n'évoque un secret
#[test]
fn connection_never_serializes_a_key_field() {
    let connection = sample_connection();
    let categories = &connection.masking.categories;
    assert!(categories.contains(&SensitiveCategory::Secret));
    let value = serde_json::to_value(&connection).unwrap();
    let top = value.as_object().unwrap();
    let mut top: Vec<&str> = top.keys().map(String::as_str).collect();
    top.sort_unstable();
    assert_eq!(top, CONNECTION_KEYS);
    let mut names = Vec::new();
    key_names(&value, &mut names);
    for name in names.iter().filter(|name| *name != "has_key") {
        let lower = name.to_lowercase();
        assert!(
            !SENSITIVE_WORDS.iter().any(|word| lower.contains(word)),
            "{name}"
        );
    }
}
