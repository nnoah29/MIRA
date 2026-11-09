#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Responses, liste des modèles : seuls les modèles listés, identifiés par `slug`.

use mira_core::ai::wire::dispatch::decode_models;
use mira_vocabulary::{AiError, WireFormat};

/// Seuls les modèles de visibilité `list` sont gardés, sous leur `slug`
#[test]
fn models_keep_listed_only_and_use_slug() {
    let body = br#"{"data":[{"slug":"a","visibility":"list","created":5},{"slug":"b","visibility":"hide"},{"id":"c","visibility":"list"}]}"#;
    let models = decode_models(WireFormat::OpenAiResponses, body).unwrap();
    let ids: Vec<&str> = models.iter().map(|model| model.id.as_str()).collect();
    assert_eq!(ids, ["a"]);
    assert_eq!(models[0].created_at, Some(5));
}

/// La racine `models` est acceptée comme `data`
#[test]
fn models_root_may_be_named_models() {
    let body = br#"{"models":[{"slug":"z","visibility":"list"}]}"#;
    assert_eq!(
        decode_models(WireFormat::OpenAiResponses, body)
            .unwrap()
            .len(),
        1
    );
}

/// Un corps sans liste est une réponse inattendue
#[test]
fn models_without_list_is_bad_response() {
    let error = decode_models(WireFormat::OpenAiResponses, b"{}").unwrap_err();
    assert_eq!(error, AiError::BadResponse);
}
