//! Ordre de consommation des comportements.

use mira_testkit::ai::{Behavior, FakeAiServer};

use crate::ai_support::client::{exchange, post_request};

/// Fabrique un comportement de statut simple avec un corps donné.
fn status(code: u16, body: &str) -> Behavior {
    Behavior::Status {
        code,
        headers: vec![],
        body: body.to_string(),
    }
}

/// Les comportements sont consommés dans l'ordre et le dernier se répète.
#[tokio::test]
async fn behaviors_are_consumed_in_order_and_the_last_repeats() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(status(500, "premier"));
    server.queue(status(200, "second"));
    let mut seen = Vec::new();
    for _ in 0..3 {
        let reply = exchange(&server.base_url(), &post_request("/", &[], "{}")).await;
        seen.push((reply.status, reply.text()));
    }
    let expected = [(500, "premier"), (200, "second"), (200, "second")];
    assert_eq!(seen, expected.map(|(c, t)| (c, t.to_string())));
    server.stop().await;
}

/// Sans comportement en file, le serveur répond 404 plutôt que de rester muet.
#[tokio::test]
async fn empty_queue_answers_404() {
    let server = FakeAiServer::start().await.unwrap();
    let reply = exchange(&server.base_url(), &post_request("/", &[], "{}")).await;
    assert_eq!(reply.status, 404);
    server.stop().await;
}

/// Models sert la liste JSON avec un type application/json.
#[tokio::test]
async fn models_serves_json() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(Behavior::Models {
        json: r#"{"data":[]}"#.into(),
    });
    let reply = exchange(
        &server.base_url(),
        "GET /v1/models HTTP/1.1\r\nHost: h\r\n\r\n",
    )
    .await;
    assert_eq!(reply.status, 200);
    assert_eq!(reply.header("content-type"), Some("application/json"));
    assert_eq!(reply.text(), r#"{"data":[]}"#);
    server.stop().await;
}
