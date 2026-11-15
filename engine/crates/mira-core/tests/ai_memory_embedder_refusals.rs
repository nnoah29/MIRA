#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Refus du service d'embeddings : un texte trop grand n'est jamais réessayable, une panne de service l'est selon sa nature.

mod ai_transport_support;

use ai_transport_support::{fast_limits, key};
use mira_core::ai::memory::{Embedder, HttpEmbedder};
use mira_testkit::ai::{Behavior, FakeAiServer};
use mira_vocabulary::error::codes::invalid_input::AI_INPUT_TOO_LARGE;

/// Un embedder branché sur ce faux serveur
fn embedder(server: &FakeAiServer) -> HttpEmbedder {
    HttpEmbedder::new(&server.base_url(), key(), "embed-1".into(), fast_limits())
}

/// Un 400, 413 ou 422 dit « texte trop grand » : erreur typée, non réessayable, sans écho du corps
#[tokio::test]
async fn too_large_input_is_typed_and_not_retryable() {
    for code in [400, 413, 422] {
        let server = FakeAiServer::start().await.unwrap();
        let body = "sk-secret détail interne".to_string();
        server.queue(Behavior::Status {
            code,
            headers: vec![],
            body,
        });
        let error = embedder(&server)
            .embed(&["long".to_string()])
            .await
            .unwrap_err();
        assert_eq!(error.code, AI_INPUT_TOO_LARGE.0, "{code}");
        assert!(!error.retryable, "{code}");
        assert!(!error.message.contains("sk-secret"), "{code}");
    }
}
