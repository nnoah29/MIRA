#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Arrêt demandé du serveur local : le processus disparaît, le modèle reste installé.

use mira_core::api::AiRequest;

mod local_chat_support;
mod local_support;
mod support_api;
use local_chat_support::fixture::LocalFixture;
use local_chat_support::read::{collect, start_for};
use local_support::runtime::RuntimeFixture;

/// Après une discussion le serveur tourne ; l'arrêt demandé le termine et le modèle reste installé
#[tokio::test]
async fn stop_runtime_stops_the_process_and_keeps_the_model() {
    let fixture = LocalFixture::start(300);
    fixture.add_connection(Some("mini"));
    let mut stream = fixture
        .engine
        .chat_stream(start_for("x", false))
        .await
        .unwrap();
    collect(&mut stream).await;
    let pid = fixture.entries("start")[0]["pid"].as_u64().unwrap();
    assert!(RuntimeFixture::alive(pid));
    fixture
        .engine
        .execute(mira_core::api::Request::Ai(AiRequest::StopLocalRuntime))
        .await
        .unwrap();
    assert!(!RuntimeFixture::alive(pid));
    assert!(fixture.root.join("models").join("mini.bin").exists());
}
