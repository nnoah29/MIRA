#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format OpenAI Chat, décodage de la liste des modèles.

use mira_core::ai::wire::openai_models::decode_models;
use mira_vocabulary::{AiError, AiModel};

/// Les identifiants et la date de création (facultative) sont relevés dans l'ordre
#[test]
fn decode_models_extracts_ids_and_created() {
    let body = br#"{"object":"list","data":[{"id":"a","created":1700000000},{"id":"b"}]}"#;
    let models = decode_models(body).unwrap();
    let a = AiModel {
        id: "a".to_string(),
        created_at: Some(1_700_000_000),
    };
    let b = AiModel {
        id: "b".to_string(),
        created_at: None,
    };
    assert_eq!(models, vec![a, b]);
}

/// Une liste vide est valide
#[test]
fn decode_models_empty_list_is_ok() {
    assert!(decode_models(br#"{"data":[]}"#).unwrap().is_empty());
}

/// Sans champ data, ou avec un corps illisible, la réponse est inattendue
#[test]
fn decode_models_without_data_field_is_bad_response() {
    let missing = decode_models(br#"{"error":{"message":"secret"}}"#).unwrap_err();
    assert_eq!(missing, AiError::BadResponse);
    assert_eq!(decode_models(b"nope").unwrap_err(), AiError::BadResponse);
}
