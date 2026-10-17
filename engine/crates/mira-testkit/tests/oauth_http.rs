#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Mini serveur HTTP/1.1 du faux serveur OAuth : routage, en-têtes, corps, requêtes invalides.

mod oauth_support;

use std::time::Duration;

use oauth_support::LIMIT;
use oauth_support::exchange::address;
use oauth_support::flow::started;
use oauth_support::http::{post_form, post_with_type, raw_exchange};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};

/// Un chemin inconnu reçoit 404 et GET sur /token reçoit 405.
#[tokio::test]
async fn unknown_path_gets_404_and_get_on_token_gets_405() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let url = address(&server);
        let request = "GET /nowhere HTTP/1.1\r\nHost: x\r\n\r\n";
        assert_eq!(raw_exchange(&url, request.as_bytes()).await.status, 404);
        let request = "GET /token HTTP/1.1\r\nHost: x\r\n\r\n";
        assert_eq!(raw_exchange(&url, request.as_bytes()).await.status, 405);
    })
    .await
    .unwrap();
}

/// Les réponses portent Content-Length, Connection: close et un type JSON.
#[tokio::test]
async fn responses_carry_length_close_and_json_type() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let reply = post_form(&address(&server), "/token", "grant_type=x").await;
        assert!(reply.headers.contains("Content-Type: application/json"));
        assert!(reply.headers.contains("Connection: close"));
        let length = format!("Content-Length: {}", reply.body.len());
        assert!(reply.headers.contains(&length), "{}", reply.headers);
    })
    .await
    .unwrap();
}

/// Un corps envoyé après les en-têtes, en plusieurs écritures, est lu selon Content-Length.
#[tokio::test]
async fn body_sent_in_a_later_write_is_read_by_content_length() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let body = "grant_type=refresh_token&refresh_token=nope&client_id=c";
        let head = format!("POST /token HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n", body.len());
        let mut stream = TcpStream::connect(address(&server)).await.unwrap();
        stream.write_all(head.as_bytes()).await.unwrap();
        sleep(Duration::from_millis(50)).await;
        stream.write_all(body.as_bytes()).await.unwrap();
        let mut text = String::new();
        stream.read_to_string(&mut text).await.unwrap();
        assert!(text.contains("invalid_grant"), "{text}");
    })
    .await
    .unwrap();
}

/// Un corps qui n'est pas un formulaire reçoit invalid_request.
#[tokio::test]
async fn non_form_content_type_gets_invalid_request() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let body = r#"{"grant_type":"refresh_token"}"#;
        let reply = post_with_type(&address(&server), "/token", "application/json", body).await;
        assert_eq!(
            (reply.status, reply.json_field("error")),
            (400, "invalid_request".into())
        );
    })
    .await
    .unwrap();
}

/// Une ligne de requête malformée reçoit 400.
#[tokio::test]
async fn malformed_request_line_gets_400() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let reply = raw_exchange(&address(&server), b"NONSENSE\r\n\r\n").await;
        assert_eq!(reply.status, 400);
    })
    .await
    .unwrap();
}
