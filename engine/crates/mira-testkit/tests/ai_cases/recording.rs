//! Enregistrement des requêtes et adresse d'écoute.

use mira_testkit::ai::{Behavior, FakeAiServer};
use tokio::net::TcpStream;

use crate::ai_support::client::{address, exchange, post_request};

/// Méthode, chemin, en-têtes en minuscules et corps sont conservés dans l'ordre d'arrivée.
#[tokio::test]
async fn records_headers_and_body() {
    let server = FakeAiServer::start().await.unwrap();
    let headers = [("Authorization", "Bearer sk-test"), ("X-Api-Key", "k-1")];
    let base = server.base_url();
    let post = post_request("/v1/chat?x=1", &headers, r#"{"a":"é"}"#);
    exchange(&base, &post).await;
    exchange(&base, "GET /v1/models HTTP/1.1\r\nHost: h\r\n\r\n").await;
    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].path, "/v1/chat?x=1");
    assert_eq!(requests[0].body, r#"{"a":"é"}"#);
    assert_eq!(requests[0].header("authorization"), Some("Bearer sk-test"));
    assert_eq!(requests[0].header("x-api-key"), Some("k-1"));
    assert!(
        requests[0]
            .headers
            .iter()
            .all(|(k, _)| k.to_lowercase() == *k)
    );
    assert_eq!(
        (requests[1].method.as_str(), requests[1].path.as_str()),
        ("GET", "/v1/models")
    );
    assert_eq!(requests[1].body, "");
    server.stop().await;
}

/// Le serveur écoute sur 127.0.0.1 seulement et la sortie de débogage ne montre aucun secret.
#[tokio::test]
async fn listens_on_loopback_only() {
    let server = FakeAiServer::start().await.unwrap();
    assert!(server.base_url().starts_with("http://127.0.0.1:"));
    assert!(server.local_addr().ip().is_loopback());
    assert!(
        TcpStream::connect(address(&server.base_url()))
            .await
            .is_ok()
    );
    let headers = [("Authorization", "Bearer sk-secret")];
    server.queue(Behavior::Hang);
    let stream = crate::ai_support::client::open_and_send(
        &server.base_url(),
        &post_request("/", &headers, "mail"),
    )
    .await;
    crate::ai_support::wait::until(|| !server.requests().is_empty()).await;
    let shown = format!("{:?} {:?}", server.requests(), server);
    assert!(
        !shown.contains("sk-secret") && !shown.contains("mail"),
        "{shown}"
    );
    drop(stream);
    server.stop().await;
}
