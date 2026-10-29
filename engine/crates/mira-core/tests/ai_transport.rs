#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Transport HTTP de l'IA : flux dans l'ordre, clé selon le descripteur, HTTPS, redirections, GET borné.

mod ai_transport_support;

use ai_transport_support::{KEY, call, descriptor, fast_limits, key, read_all};
use mira_core::ai::transport::{get_bounded, post_stream};
use mira_testkit::ai::{Behavior, FakeAiServer, openai_stream};
use mira_vocabulary::{AiError, AuthScheme};

/// Le flux diffusé arrive en entier et dans l'ordre ; la requête porte méthode, chemin et corps
#[tokio::test]
async fn post_stream_delivers_chunks_in_order() {
    let server = FakeAiServer::start().await.unwrap();
    let Behavior::Stream { chunks, .. } = openai_stream(&["Bon", "jour", " !"], Some((3, 2)))
    else {
        panic!("flux attendu");
    };
    server.queue(Behavior::Stream {
        chunks: chunks.clone(),
        chunk_delay_ms: 20,
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
    assert_eq!(read_all(&mut body).await.unwrap(), chunks.concat());
    assert_eq!(body.next_chunk().await.unwrap(), None);
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].path, "/chat/completions");
    assert_eq!(requests[0].body, call().body);
}

/// La clé part en `Authorization: Bearer` ou en `x-api-key` selon le descripteur, jamais les deux
#[tokio::test]
async fn key_is_sent_as_bearer_or_x_api_key_per_descriptor() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(openai_stream(&["ok"], None));
    for auth in [AuthScheme::Bearer, AuthScheme::XApiKey] {
        let descriptor = descriptor(auth);
        let mut body = post_stream(
            &server.base_url(),
            &descriptor,
            &key(),
            &call(),
            &fast_limits(),
        )
        .await
        .unwrap();
        read_all(&mut body).await.unwrap();
    }
    let requests = server.requests();
    let bearer = format!("Bearer {KEY}");
    assert_eq!(requests[0].header("authorization"), Some(bearer.as_str()));
    assert_eq!(requests[0].header("x-api-key"), None);
    assert_eq!(requests[1].header("x-api-key"), Some(KEY));
    assert_eq!(requests[1].header("authorization"), None);
}

/// Hors boucle locale, une URL non HTTPS ou un chemin qui déborde sur l'hôte est refusé avant tout réseau
#[tokio::test]
async fn https_is_required_outside_loopback() {
    let server = FakeAiServer::start().await.unwrap();
    let descriptor = descriptor(AuthScheme::Bearer);
    let limits = fast_limits();
    for base in [
        "http://exemple.invalid",
        "ftp://127.0.0.1",
        "pas une url",
        "",
    ] {
        let streamed = post_stream(base, &descriptor, &key(), &call(), &limits).await;
        assert_eq!(streamed.err(), Some(AiError::UrlInvalid), "{base}");
        let listed = get_bounded(base, &descriptor, &key(), "/models", &limits).await;
        assert_eq!(listed.err(), Some(AiError::UrlInvalid), "{base}");
    }
    let mut sneaky = call();
    sneaky.path = "@exemple.invalid/chat".to_string();
    let streamed = post_stream(&server.base_url(), &descriptor, &key(), &sneaky, &limits).await;
    assert_eq!(streamed.err(), Some(AiError::UrlInvalid));
    let listed = get_bounded(&server.base_url(), &descriptor, &key(), "models", &limits).await;
    assert_eq!(listed.err(), Some(AiError::UrlInvalid));
    assert!(server.requests().is_empty());
    let secure = "https://api.exemple.invalid";
    let streamed = post_stream(secure, &descriptor, &key(), &sneaky, &limits).await;
    assert_eq!(streamed.err(), Some(AiError::UrlInvalid));
    let listed = get_bounded(secure, &descriptor, &key(), "@autre.invalid/m", &limits).await;
    assert_eq!(listed.err(), Some(AiError::UrlInvalid));
}

/// Une redirection n'est jamais suivie : la cible ne reçoit rien, donc jamais la clé
#[tokio::test]
async fn redirect_is_not_followed_and_key_not_resent() {
    let first = FakeAiServer::start().await.unwrap();
    let second = FakeAiServer::start().await.unwrap();
    second.queue(openai_stream(&["volé"], None));
    first.queue(Behavior::Redirect {
        to: format!("{}/chat/completions", second.base_url()),
    });
    let descriptor = descriptor(AuthScheme::XApiKey);
    let limits = fast_limits();
    let streamed = post_stream(&first.base_url(), &descriptor, &key(), &call(), &limits).await;
    assert_eq!(streamed.err(), Some(AiError::BadResponse));
    let listed = get_bounded(&first.base_url(), &descriptor, &key(), "/models", &limits).await;
    assert_eq!(listed.err(), Some(AiError::BadResponse));
    assert_eq!(first.requests().len(), 2);
    assert!(second.requests().is_empty());
}

/// La liste des modèles est lue en GET avec `accept`, les en-têtes statiques et la clé
#[tokio::test]
async fn get_bounded_sends_accept_static_headers_and_key() {
    let server = FakeAiServer::start().await.unwrap();
    let json = r#"{"data":[{"id":"m1"}]}"#;
    server.queue(Behavior::Models {
        json: json.to_string(),
    });
    let descriptor = descriptor(AuthScheme::Bearer);
    let body = get_bounded(
        &server.base_url(),
        &descriptor,
        &key(),
        "/models",
        &fast_limits(),
    )
    .await
    .unwrap();
    assert_eq!(body, json.as_bytes());
    let request = &server.requests()[0];
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/models");
    assert_eq!(request.header("accept"), Some("application/json"));
    assert_eq!(request.header("x-test"), Some("1"));
    let bearer = format!("Bearer {KEY}");
    assert_eq!(request.header("authorization"), Some(bearer.as_str()));
}
