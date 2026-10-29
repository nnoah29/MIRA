#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Transport HTTP de l'IA : traduction des statuts HTTP et des pannes de connexion en `AiError`.

mod ai_transport_support;

use ai_transport_support::{call, descriptor, fast_limits, key};
use mira_core::ai::transport::{get_bounded, map_status, post_stream};
use mira_testkit::ai::{Behavior, FakeAiServer};
use mira_vocabulary::{AiError, AuthScheme};

/// Réponse d'échec avec ce code et ces en-têtes.
fn status(code: u16, headers: &[(&str, &str)]) -> Behavior {
    Behavior::Status {
        code,
        headers: headers
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect(),
        body: r#"{"error":"x"}"#.to_string(),
    }
}

/// Chaque statut donne la variante prévue
#[test]
fn map_status_table() {
    let table = [
        (401, AiError::KeyRejected),
        (403, AiError::KeyRejected),
        (404, AiError::ModelNotFound),
        (
            429,
            AiError::RateLimited {
                retry_after_secs: None,
            },
        ),
        (400, AiError::InputTooLarge),
        (413, AiError::InputTooLarge),
        (422, AiError::InputTooLarge),
        (500, AiError::Unavailable),
        (502, AiError::Unavailable),
        (599, AiError::Unavailable),
        (402, AiError::BadResponse),
        (405, AiError::BadResponse),
        (418, AiError::BadResponse),
        (302, AiError::BadResponse),
        (200, AiError::BadResponse),
    ];
    for (code, expected) in table {
        assert_eq!(map_status(code, None), expected, "{code}");
    }
    assert_eq!(map_status(503, Some("5")), AiError::Unavailable);
}

/// `retry-after` est lu en secondes, plafonné à une heure ; une date ou une valeur illisible ne donne rien
#[test]
fn map_status_reads_retry_after_seconds() {
    let table = [
        (Some("7"), Some(7)),
        (Some(" 12 "), Some(12)),
        (Some("0"), Some(0)),
        (None, None),
        (Some(""), None),
        (Some("Wed, 21 Oct 2015 07:28:00 GMT"), None),
        (Some("-3"), None),
        (Some("+3"), None),
        (Some("1.5"), None),
        (Some("3600"), Some(3600)),
        (Some("86400"), Some(3600)),
        (Some("999999999999999999999999"), Some(3600)),
    ];
    for (header, secs) in table {
        let expected = AiError::RateLimited {
            retry_after_secs: secs,
        };
        assert_eq!(map_status(429, header), expected, "{header:?}");
    }
}

/// Un statut d'échec du flux est traduit, avec le délai `retry-after` du serveur
#[tokio::test]
async fn post_stream_maps_http_status() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(status(401, &[]));
    server.queue(status(429, &[("Retry-After", "7")]));
    server.queue(status(503, &[]));
    let descriptor = descriptor(AuthScheme::Bearer);
    let expected = [
        AiError::KeyRejected,
        AiError::RateLimited {
            retry_after_secs: Some(7),
        },
        AiError::Unavailable,
    ];
    for error in expected {
        let base = server.base_url();
        let streamed = post_stream(&base, &descriptor, &key(), &call(), &fast_limits()).await;
        assert_eq!(streamed.err(), Some(error));
    }
}

/// Un statut d'échec de la liste des modèles est traduit de la même façon
#[tokio::test]
async fn get_bounded_maps_http_status() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(status(404, &[]));
    server.queue(status(403, &[]));
    let descriptor = descriptor(AuthScheme::XApiKey);
    for error in [AiError::ModelNotFound, AiError::KeyRejected] {
        let base = server.base_url();
        let listed = get_bounded(&base, &descriptor, &key(), "/models", &fast_limits()).await;
        assert_eq!(listed.err(), Some(error));
    }
}

/// Un service injoignable est indisponible
#[tokio::test]
async fn unreachable_service_is_unavailable() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    drop(listener);
    let descriptor = descriptor(AuthScheme::Bearer);
    let streamed = post_stream(&base, &descriptor, &key(), &call(), &fast_limits()).await;
    assert_eq!(streamed.err(), Some(AiError::Unavailable));
    let listed = get_bounded(&base, &descriptor, &key(), "/models", &fast_limits()).await;
    assert_eq!(listed.err(), Some(AiError::Unavailable));
}
