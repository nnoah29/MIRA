#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Format Messages, décodage de la liste des modèles.

use mira_core::ai::wire::anthropic_models::decode_models;
use mira_vocabulary::{AiError, AiModel};

/// Les identifiants et la date RFC 3339 (convertie en secondes Unix) sont relevés dans l'ordre
#[test]
fn decode_models_extracts_ids_and_created_at() {
    let body = br#"{"data":[
        {"id":"a","created_at":"2025-02-19T00:00:00Z"},
        {"id":"b","created_at":"2023-11-14T23:13:20.5+01:00"},
        {"id":"c","created_at":"hier"},
        {"id":"d"}],"has_more":false}"#;
    let created = |id: &str, created_at| AiModel {
        id: id.to_string(),
        created_at,
    };
    let expected = vec![
        created("a", Some(1_739_923_200)),
        created("b", Some(1_700_000_000)),
        created("c", None),
        created("d", None),
    ];
    assert_eq!(decode_models(body).unwrap(), expected);
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
