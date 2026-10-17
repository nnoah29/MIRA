#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réponse du point d'accès de jetons : lue par morceaux et plafonnée à 1 Mo (audit 14).

mod support_auth;

use support_auth::LIMIT;
use support_auth::canned::exchange_failure;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::timeout;

const CHUNK: usize = 64 * 1024;
const TOTAL: usize = 64 * 1024 * 1024;

/// Sert une réponse sans `Content-Length` de 64 Mo ; rend le nombre d'octets réellement acceptés
async fn serve_endless() -> (String, JoinHandle<usize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        drain_request(&mut stream).await;
        let head = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n";
        stream.write_all(head.as_bytes()).await.unwrap();
        let block = vec![b'a'; CHUNK];
        let mut written = 0;
        while written < TOTAL && stream.write_all(&block).await.is_ok() {
            written += CHUNK;
        }
        written
    });
    (format!("http://127.0.0.1:{port}/token"), handle)
}

/// Lit la requête jusqu'à la fin de son en-tête et de son corps
async fn drain_request(stream: &mut TcpStream) {
    let mut data = Vec::new();
    let mut chunk = [0_u8; 1024];
    while !data.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = stream.read(&mut chunk).await.unwrap();
        data.extend_from_slice(&chunk[..read]);
    }
    let mut body = [0_u8; 1024];
    let _ = timeout(std::time::Duration::from_millis(50), stream.read(&mut body)).await;
}

/// Une réponse démesurée coupe la connexion après 1 Mo au lieu d'être entièrement lue
#[tokio::test]
async fn regression_oversized_token_response_is_cut_off_early() {
    timeout(LIMIT, async {
        let (url, server) = serve_endless().await;
        let error = exchange_failure(url).await;
        assert_eq!(error.code, "server.protocol");
        let written = server.await.unwrap();
        assert!(written < TOTAL / 2, "{written} octets envoyés");
    })
    .await
    .unwrap();
}
