//! Comportements d'échec : statut, silence, coupure, redirection, taille excessive, déconnexion.

use std::time::Duration;

use mira_testkit::ai::{Behavior, FakeAiServer};
use tokio::io::AsyncReadExt;
use tokio::time::timeout;

use crate::ai_support::client::{exchange, open_and_send, post_request};

/// Status renvoie le code, les en-têtes donnés et le corps.
#[tokio::test]
async fn status_behavior_returns_code_and_headers() {
    let server = FakeAiServer::start().await.unwrap();
    let headers = vec![("retry-after".to_string(), "7".to_string())];
    server.queue(Behavior::Status {
        code: 429,
        headers,
        body: r#"{"error":"slow"}"#.into(),
    });
    let reply = exchange(&server.base_url(), &post_request("/", &[], "{}")).await;
    assert_eq!(reply.status, 429);
    assert_eq!(reply.header("retry-after"), Some("7"));
    assert_eq!(reply.header("content-type"), Some("application/json"));
    assert_eq!(reply.text(), r#"{"error":"slow"}"#);
    server.stop().await;
}

/// Hang ne répond jamais ; l'arrêt du serveur ferme la connexion.
#[tokio::test]
async fn hang_never_answers_until_stopped() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Hang);
    let mut stream = open_and_send(&server.base_url(), &post_request("/", &[], "{}")).await;
    let mut byte = [0u8; 1];
    let silent = timeout(Duration::from_millis(300), stream.read(&mut byte)).await;
    assert!(silent.is_err(), "le serveur a répondu");
    server.stop().await;
    let after = timeout(Duration::from_secs(5), stream.read(&mut byte))
        .await
        .unwrap();
    assert!(matches!(after, Ok(0) | Err(_)));
}

/// DropAfter envoie les morceaux puis coupe sans le morceau final.
#[tokio::test]
async fn drop_after_closes_mid_stream() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::DropAfter {
        chunks: vec![b"data: 1\n\n".to_vec()],
    });
    let reply = exchange(&server.base_url(), &post_request("/", &[], "{}")).await;
    assert_eq!(reply.status, 200);
    assert_eq!(reply.text(), "data: 1\n\n");
    assert!(!reply.complete);
    server.stop().await;
}

/// Redirect renvoie 302 avec l'en-tête Location.
#[tokio::test]
async fn redirect_returns_302_location() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Redirect {
        to: "http://127.0.0.1:1/ailleurs".into(),
    });
    let reply = exchange(&server.base_url(), &post_request("/", &[], "{}")).await;
    assert_eq!(reply.status, 302);
    assert_eq!(
        reply.header("location"),
        Some("http://127.0.0.1:1/ailleurs")
    );
    server.stop().await;
}

/// Oversize diffuse exactement le nombre d'octets demandé, en SSE valide.
#[tokio::test]
async fn oversize_streams_requested_bytes() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Oversize { bytes: 100_003 });
    let reply = exchange(&server.base_url(), &post_request("/", &[], "{}")).await;
    assert_eq!(reply.status, 200);
    assert!(reply.complete);
    assert_eq!(reply.body.len(), 100_003);
    assert!(reply.header("content-length").is_none());
    server.stop().await;
}
