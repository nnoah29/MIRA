#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Requête et réponse de l'API d'embeddings compatible OpenAI : forme, ordre, dimensions et refus.

use mira_core::ai::memory::{embeddings_request, parse_embeddings};
use mira_vocabulary::error::codes::server::AI_BAD_RESPONSE;
use serde_json::json;

/// La requête nomme le modèle et porte tous les textes dans l'ordre
#[test]
fn request_carries_model_and_inputs() {
    let body = embeddings_request("m1", &["a".to_string(), "b".to_string()]);
    assert_eq!(body, json!({"model": "m1", "input": ["a", "b"]}));
}

/// Les vecteurs sont rendus dans l'ordre de `index`, même si le serveur les mélange
#[test]
fn parse_orders_by_index() {
    let body = json!({"data": [
        {"index": 1, "embedding": [0.0, 1.0]},
        {"index": 0, "embedding": [1.0, 0.0]}
    ]})
    .to_string();
    let vectors = parse_embeddings(&body, 2).unwrap();
    assert_eq!(vectors, vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
}

/// Des dimensions différentes d'un texte à l'autre sont refusées
#[test]
fn parse_rejects_inconsistent_dimensions() {
    let body = json!({"data": [
        {"index": 0, "embedding": [1.0, 0.0]},
        {"index": 1, "embedding": [1.0]}
    ]})
    .to_string();
    let error = parse_embeddings(&body, 2).unwrap_err();
    assert_eq!(error.code, AI_BAD_RESPONSE.0);
}

/// Un nombre de vecteurs différent du nombre de textes est refusé
#[test]
fn parse_rejects_wrong_count() {
    let body = json!({"data": [{"index": 0, "embedding": [1.0]}]}).to_string();
    let error = parse_embeddings(&body, 2).unwrap_err();
    assert_eq!(error.code, AI_BAD_RESPONSE.0);
}

/// Un vecteur vide, un corps illisible ou un nombre non fini sont refusés sans citer le corps
#[test]
fn parse_rejects_empty_vector_and_garbage() {
    let empty = json!({"data": [{"index": 0, "embedding": []}]}).to_string();
    assert!(parse_embeddings(&empty, 1).is_err());
    let error = parse_embeddings("<html>secret-token</html>", 1).unwrap_err();
    assert!(!error.message.contains("secret-token"));
}

/// Des index répétés ou qui ne forment pas 0..n sont refusés : les vecteurs iraient aux mauvais textes
#[test]
fn parse_rejects_duplicate_or_gapped_indexes() {
    for indexes in [[0, 0], [1, 2]] {
        let body = json!({"data": [
            {"index": indexes[0], "embedding": [1.0]},
            {"index": indexes[1], "embedding": [2.0]}
        ]})
        .to_string();
        let error = parse_embeddings(&body, 2).unwrap_err();
        assert_eq!(error.code, AI_BAD_RESPONSE.0);
    }
}
