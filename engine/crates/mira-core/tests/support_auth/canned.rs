//! Faux point d'accès de jetons qui répond une seule fois par une réponse HTTP imposée.

use mira_core::auth::{AuthorizationCode, exchange_code};
use mira_core::ports::SecretValue;
use mira_vocabulary::MiraError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::flow::clock;

/// Lance un serveur à usage unique et renvoie l'adresse `http://127.0.0.1:port/token`
pub async fn serve_once(status_line: &'static str, body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        read_request(&mut stream).await;
        let reply = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(reply.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();
    });
    format!("http://127.0.0.1:{port}/token")
}

/// Tente un échange de code auprès de cette adresse et renvoie l'erreur attendue
pub async fn exchange_failure(token_url: String) -> MiraError {
    let code = AuthorizationCode::new("c");
    let verifier = SecretValue::new("v".repeat(43));
    let redirect = "http://127.0.0.1:1/cb";
    exchange_code(&token_url, "client", &code, &verifier, redirect, &clock())
        .await
        .unwrap_err()
}

/// Lit l'en-tête de la requête puis son corps, selon `Content-Length`
async fn read_request(stream: &mut TcpStream) {
    let mut data = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream.read(&mut chunk).await.unwrap();
        data.extend_from_slice(&chunk[..read]);
        let text = String::from_utf8_lossy(&data).into_owned();
        let Some((head, body)) = text.split_once("\r\n\r\n") else {
            continue;
        };
        let wanted = head
            .lines()
            .find_map(|line| {
                line.to_ascii_lowercase()
                    .strip_prefix("content-length:")
                    .map(|v| v.trim().parse::<usize>().unwrap())
            })
            .unwrap_or(0);
        if body.len() >= wanted || read == 0 {
            return;
        }
    }
}
