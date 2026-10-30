#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Transport HTTP de l'IA face aux pannes : délais, coupure, plafonds de taille, abandon du flux.

mod ai_transport_support;

use std::time::{Duration, Instant};

use ai_transport_support::{call, descriptor, fast_limits, key, read_all, wait_closed_by_client};
use mira_core::ai::transport::{get_bounded, post_stream};
use mira_testkit::ai::{Behavior, FakeAiServer};
use mira_vocabulary::{AiError, AuthScheme};

/// Un serveur muet après la requête dépasse le délai du premier octet : service indisponible
#[tokio::test]
async fn first_byte_timeout_maps_to_unavailable() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Hang);
    let descriptor = descriptor(AuthScheme::Bearer);
    let limits = fast_limits();
    let started = Instant::now();
    let streamed = post_stream(&server.base_url(), &descriptor, &key(), &call(), &limits).await;
    assert_eq!(streamed.err(), Some(AiError::Unavailable));
    let listed = get_bounded(&server.base_url(), &descriptor, &key(), "/models", &limits).await;
    assert_eq!(listed.err(), Some(AiError::Unavailable));
    assert!(started.elapsed() < Duration::from_secs(3));
}

/// Un silence entre deux morceaux au-delà du délai d'inactivité arrête le flux
#[tokio::test]
async fn idle_timeout_between_chunks() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Stream {
        chunks: vec![b"data: a\n\n".to_vec(), b"data: b\n\n".to_vec()],
        chunk_delay_ms: 3000,
    });
    let descriptor = descriptor(AuthScheme::Bearer);
    let limits = fast_limits();
    let mut body = post_stream(&server.base_url(), &descriptor, &key(), &call(), &limits)
        .await
        .unwrap();
    assert_eq!(
        body.next_chunk().await.unwrap(),
        Some(b"data: a\n\n".to_vec())
    );
    let started = Instant::now();
    assert_eq!(body.next_chunk().await, Err(AiError::Unavailable));
    assert!(started.elapsed() < Duration::from_millis(2000));
}

/// Une connexion coupée au milieu du flux donne service indisponible, après les morceaux reçus
#[tokio::test]
async fn connection_drop_mid_stream_is_unavailable() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::DropAfter {
        chunks: vec![b"data: a\n\n".to_vec()],
    });
    let descriptor = descriptor(AuthScheme::Bearer);
    let mut body = post_stream(
        &server.base_url(),
        &descriptor,
        &key(),
        &call(),
        &fast_limits(),
    )
    .await
    .unwrap();
    assert_eq!(
        body.next_chunk().await.unwrap(),
        Some(b"data: a\n\n".to_vec())
    );
    assert_eq!(body.next_chunk().await, Err(AiError::Unavailable));
}

/// Un flux qui dépasse le plafond total est une réponse invalide ; exactement le plafond passe
#[tokio::test]
async fn oversize_stream_is_bad_response() {
    let server = FakeAiServer::start().await.unwrap();
    let mut limits = fast_limits();
    limits.max_stream = 64 * 1024;
    server.queue(Behavior::Oversize { bytes: 64 * 1024 });
    server.queue(Behavior::Oversize {
        bytes: 64 * 1024 + 1,
    });
    let descriptor = descriptor(AuthScheme::Bearer);
    let base = server.base_url();
    let mut exact = post_stream(&base, &descriptor, &key(), &call(), &limits)
        .await
        .unwrap();
    assert_eq!(read_all(&mut exact).await.unwrap().len(), 64 * 1024);
    let mut over = post_stream(&base, &descriptor, &key(), &call(), &limits)
        .await
        .unwrap();
    assert_eq!(read_all(&mut over).await, Err(AiError::BadResponse));
}

/// Un corps de liste au-delà du plafond, découpé ou annoncé par sa longueur, est une réponse invalide
#[tokio::test]
async fn oversize_body_in_get_bounded() {
    let server = FakeAiServer::start().await.unwrap();
    let mut limits = fast_limits();
    limits.max_body = 16 * 1024;
    server.queue(Behavior::Oversize {
        bytes: 16 * 1024 + 1,
    });
    server.queue(Behavior::Models {
        json: format!(r#"{{"data":"{}"}}"#, "x".repeat(32 * 1024)),
    });
    let descriptor = descriptor(AuthScheme::Bearer);
    let base = server.base_url();
    for _ in 0..2 {
        let listed = get_bounded(&base, &descriptor, &key(), "/models", &limits).await;
        assert_eq!(listed.err(), Some(AiError::BadResponse));
    }
}

/// Abandonner le flux en cours ferme la connexion ; le serveur le constate
#[tokio::test]
async fn dropping_stream_body_closes_connection() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Stream {
        chunks: vec![b"data: a\n\n".to_vec(); 50],
        chunk_delay_ms: 100,
    });
    let descriptor = descriptor(AuthScheme::Bearer);
    let mut body = post_stream(
        &server.base_url(),
        &descriptor,
        &key(),
        &call(),
        &fast_limits(),
    )
    .await
    .unwrap();
    assert!(body.next_chunk().await.unwrap().is_some());
    drop(body);
    assert_eq!(wait_closed_by_client(&server).await, 1);
}
