#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Types de la confidentialité de l'IA : formes sur le fil, valeurs par défaut, journal sans contenu, codes.

use mira_vocabulary::error::codes::{self, incompatible_state, invalid_input};
use mira_vocabulary::{
    AiError, ChatDelta, ChatRequest, ContentKind, MaskingMode, MaskingSettings, MiraError,
    SensitiveCategory,
};
use serde_json::json;

/// Chaque type de contenu s'écrit en `snake_case` et se relit
#[test]
fn content_kind_serializes_snake_case() {
    let kinds = [
        (ContentKind::MailMessage, "mail_message"),
        (ContentKind::MailList, "mail_list"),
        (ContentKind::Attachment, "attachment"),
        (ContentKind::ToolResult, "tool_result"),
    ];
    for (kind, name) in kinds {
        assert_eq!(serde_json::to_value(kind).unwrap(), json!(name));
        assert_eq!(kind.as_str(), name);
        assert_eq!(
            serde_json::from_value::<ContentKind>(json!(name)).unwrap(),
            kind
        );
    }
}

/// Une requête sans champ `context` se relit avec un contexte vide
#[test]
fn chat_request_context_defaults_to_empty() {
    let raw = json!({
        "model": null,
        "system": null,
        "messages": [],
        "max_output_tokens": null,
        "temperature": null
    });
    let request: ChatRequest = serde_json::from_value(raw).unwrap();
    assert!(request.context.is_empty());
}

/// Distante : masquage actif sur quatre catégories ; locale : coupé, mêmes catégories
#[test]
fn masking_defaults_remote_and_local() {
    let categories = vec![
        SensitiveCategory::Iban,
        SensitiveCategory::Card,
        SensitiveCategory::Secret,
        SensitiveCategory::Otp,
    ];
    let remote = MaskingSettings::remote_default();
    assert_eq!(remote.mode, MaskingMode::Mask);
    assert_eq!(remote.categories, categories);
    let local = MaskingSettings::local_default();
    assert_eq!(local.mode, MaskingMode::Off);
    assert_eq!(local.categories, categories);
    assert_eq!(MaskingSettings::default(), remote);
}

/// Le delta de masquage porte son type en `snake_case` et le nombre de valeurs masquées
#[test]
fn masked_delta_serializes_with_type_tag() {
    let delta = ChatDelta::Masked { count: 3 };
    let value = serde_json::to_value(&delta).unwrap();
    assert_eq!(value, json!({"type": "masked", "count": 3}));
    assert_eq!(serde_json::from_value::<ChatDelta>(value).unwrap(), delta);
}

/// Les deux nouveaux codes sont au catalogue une seule fois et `ContentNotAllowed` se convertit
#[test]
fn new_error_codes_are_registered_and_mapped() {
    for code in [
        incompatible_state::AI_CONTENT_NOT_ALLOWED,
        invalid_input::AI_JOURNAL_LIMIT,
    ] {
        assert_eq!(codes::ALL.iter().filter(|c| **c == code).count(), 1);
    }
    assert_eq!(
        incompatible_state::AI_CONTENT_NOT_ALLOWED.0,
        "incompatible_state.ai_content_not_allowed"
    );
    assert_eq!(
        invalid_input::AI_JOURNAL_LIMIT.0,
        "invalid_input.ai_journal_limit"
    );
    let mira: MiraError = AiError::ContentNotAllowed.into();
    assert_eq!(mira.code, incompatible_state::AI_CONTENT_NOT_ALLOWED.0);
    assert!(mira.code.starts_with(mira.category.as_str()));
    assert!(!mira.retryable);
}
