#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Embedder HTTP compatible OpenAI : requête, vecteurs rendus, refus d'une réponse incohérente, aucune fuite de clé.

mod ai_transport_support;

use ai_transport_support::{KEY, fast_limits, key};
use mira_core::ai::memory::{Embedder, HttpEmbedder};
use mira_testkit::ai::{Behavior, FakeAiServer};
use mira_vocabulary::error::codes::server::AI_BAD_RESPONSE;
use serde_json::json;

/// Réponse 200 JSON de ce corps
fn ok(body: serde_json::Value) -> Behavior {
    Behavior::Status {
        code: 200,
        headers: vec![],
        body: body.to_string(),
    }
}

/// Un embedder branché sur ce faux serveur
fn embedder(server: &FakeAiServer) -> HttpEmbedder {
    HttpEmbedder::new(&server.base_url(), key(), "embed-1".into(), fast_limits())
}

/// Un vecteur par texte ; la requête vise `/v1/embeddings` avec la clé en Bearer
#[tokio::test]
async fn embed_returns_one_vector_per_text() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(ok(json!({"data": [
        {"index": 0, "embedding": [1.0, 0.0]},
        {"index": 1, "embedding": [0.0, 1.0]}
    ]})));
    let texts = vec!["a".to_string(), "b".to_string()];
    let vectors = embedder(&server).embed(&texts).await.unwrap();
    assert_eq!(vectors, vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
    let seen = &server.requests()[0];
    assert_eq!(seen.path, "/v1/embeddings");
    assert_eq!(
        seen.header("authorization"),
        Some(format!("Bearer {KEY}").as_str())
    );
    assert_eq!(
        seen.body,
        json!({"model": "embed-1", "input": ["a", "b"]}).to_string()
    );
}

/// Des dimensions incohérentes donnent `server.ai_bad_response`
#[tokio::test]
async fn embed_rejects_inconsistent_dimensions() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(ok(json!({"data": [
        {"index": 0, "embedding": [1.0, 0.0]},
        {"index": 1, "embedding": [1.0]}
    ]})));
    let texts = vec!["a".to_string(), "b".to_string()];
    let error = embedder(&server).embed(&texts).await.unwrap_err();
    assert_eq!(error.code, AI_BAD_RESPONSE.0);
}

/// Un refus du serveur est traduit sans jamais citer la clé
#[tokio::test]
async fn embed_maps_server_error_without_leaking_key() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Status {
        code: 401,
        headers: vec![],
        body: format!("{{\"error\":\"{KEY}\"}}"),
    });
    let error = embedder(&server)
        .embed(&["a".to_string()])
        .await
        .unwrap_err();
    assert!(!format!("{error:?}").contains(KEY));
}

/// Sans texte, aucune requête n'est émise
#[tokio::test]
async fn embed_empty_input_makes_no_request() {
    let server = FakeAiServer::start().await.unwrap();
    let vectors = embedder(&server).embed(&[]).await.unwrap();
    assert!(vectors.is_empty());
    assert!(server.requests().is_empty());
}
