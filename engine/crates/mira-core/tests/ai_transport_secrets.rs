#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Transport HTTP de l'IA et secrets : clé jamais reprise, corps d'erreur jamais recopié, en-têtes exacts.

mod ai_transport_support;

use ai_transport_support::{KEY, call, descriptor, fast_limits, key, read_all};
use mira_core::ai::transport::{get_bounded, post_stream};
use mira_core::ports::SecretValue;
use mira_testkit::ai::{Behavior, FakeAiServer, openai_stream};
use mira_vocabulary::{AiError, AiProviderDescriptor, AuthScheme, MiraError};

/// Texte de débogage et d'affichage d'une erreur, côté IA et côté contrat.
fn rendered(error: &AiError) -> String {
    let mira: MiraError = error.clone().into();
    format!("{error:?} {error} {mira:?} {}", mira.message)
}

/// Le corps d'erreur du service n'apparaît dans aucune erreur, ni d'IA ni du contrat
#[tokio::test]
async fn error_body_is_never_copied() {
    let server = FakeAiServer::start().await.unwrap();
    for code in [400, 401, 404, 429, 500, 418] {
        server.queue(Behavior::Status {
            code,
            headers: Vec::new(),
            body: r#"{"error":{"message":"SECRET-PROMPT-TEXT"}}"#.to_string(),
        });
    }
    let descriptor = descriptor(AuthScheme::Bearer);
    let base = server.base_url();
    for index in 0..6 {
        let error = if index % 2 == 0 {
            post_stream(&base, &descriptor, &key(), &call(), &fast_limits())
                .await
                .unwrap_err()
        } else {
            get_bounded(&base, &descriptor, &key(), "/models", &fast_limits())
                .await
                .unwrap_err()
        };
        assert!(
            !rendered(&error).contains("SECRET-PROMPT-TEXT"),
            "{error:?}"
        );
    }
}

/// Rien de ce qui sort du module (erreurs, flux, limites) ne montre la clé
#[tokio::test]
async fn key_never_in_debug_or_error_text() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(openai_stream(&["ok"], None));
    server.queue(Behavior::Status {
        code: 401,
        headers: Vec::new(),
        body: format!(r#"{{"error":"bad key {KEY}"}}"#),
    });
    server.queue(Behavior::Hang);
    let descriptor = descriptor(AuthScheme::Bearer);
    let limits = fast_limits();
    let base = server.base_url();
    let mut body = post_stream(&base, &descriptor, &key(), &call(), &limits)
        .await
        .unwrap();
    let mut seen = format!("{body:?} {limits:?}");
    read_all(&mut body).await.unwrap();
    seen.push_str(&format!("{body:?}"));
    for base in [base.as_str(), base.as_str(), "http://exemple.invalid"] {
        seen.push_str(&failure_text(base, &descriptor).await);
    }
    assert!(!seen.contains(KEY), "{seen}");
    assert!(!seen.contains("SECRET-KEY"), "{seen}");
}

/// Texte de l'erreur rendue par un envoi qui doit échouer vers cette adresse.
async fn failure_text(base: &str, descriptor: &AiProviderDescriptor) -> String {
    let streamed = post_stream(base, descriptor, &key(), &call(), &fast_limits()).await;
    rendered(&streamed.unwrap_err())
}

/// Une clé impossible à placer dans un en-tête est refusée sans réseau et sans être reprise
#[tokio::test]
async fn key_with_invalid_header_characters_is_refused_without_echo() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(openai_stream(&["ok"], None));
    let base = server.base_url();
    for auth in [AuthScheme::Bearer, AuthScheme::XApiKey] {
        let descriptor = descriptor(auth);
        let bad = SecretValue::new("sk-INJECTED\r\nx-evil: 1");
        let streamed = post_stream(&base, &descriptor, &bad, &call(), &fast_limits()).await;
        let error = streamed.unwrap_err();
        assert_eq!(error, AiError::KeyRejected);
        assert!(!rendered(&error).contains("INJECTED"));
        let listed = get_bounded(&base, &descriptor, &bad, "/models", &fast_limits()).await;
        assert_eq!(listed.err(), Some(AiError::KeyRejected));
    }
    assert!(server.requests().is_empty());
}

/// La requête diffusée porte exactement les en-têtes de l'appel plus celui d'authentification
#[tokio::test]
async fn requests_carry_exactly_call_headers_plus_auth() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(openai_stream(&["ok"], None));
    let descriptor = descriptor(AuthScheme::XApiKey);
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
    let request = &server.requests()[0];
    let mut sent: Vec<(String, String)> = request
        .headers
        .iter()
        .filter(|(name, _)| name != "host" && name != "content-length")
        .cloned()
        .collect();
    sent.sort();
    let mut expected = call().headers;
    expected.push(("x-api-key".to_string(), KEY.to_string()));
    expected.sort();
    assert_eq!(sent, expected);
}
