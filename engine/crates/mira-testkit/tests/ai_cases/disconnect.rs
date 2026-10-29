//! Détection de la fermeture de connexion par le client, pour tester l'annulation.

use mira_testkit::ai::{Behavior, FakeAiServer};
use tokio::io::AsyncReadExt;

use crate::ai_support::client::{exchange, open_and_send, post_request};
use crate::ai_support::wait::until;

/// La fermeture de la connexion par le client en plein flux est comptée, pas une fin normale.
#[tokio::test]
async fn client_disconnect_is_counted() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Status {
        code: 200,
        headers: vec![],
        body: "ok".into(),
    });
    server.queue(Behavior::Stream {
        chunks: vec![b"data: 1\n\n".to_vec(); 50],
        chunk_delay_ms: 100,
    });
    exchange(&server.base_url(), &post_request("/", &[], "{}")).await;
    assert_eq!(server.connections_closed_by_client(), 0);
    let mut stream = open_and_send(&server.base_url(), &post_request("/", &[], "{}")).await;
    let mut first = [0u8; 16];
    stream.read_exact(&mut first).await.unwrap();
    drop(stream);
    until(|| server.connections_closed_by_client() == 1).await;
    server.stop().await;
}

/// Un client qui part pendant un silence total (Hang) est aussi compté.
#[tokio::test]
async fn disconnect_during_hang_is_counted() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Hang);
    let stream = open_and_send(&server.base_url(), &post_request("/", &[], "{}")).await;
    until(|| !server.requests().is_empty()).await;
    assert_eq!(server.connections_closed_by_client(), 0);
    drop(stream);
    until(|| server.connections_closed_by_client() == 1).await;
    server.stop().await;
}
