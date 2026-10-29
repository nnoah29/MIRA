//! Client HTTP/1.1 brut : écrit une requête puis lit la réponse jusqu'à la fermeture.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::LIMIT;
use super::reply::{Reply, parse_reply};

/// Adresse `hôte:port` d'une URL de base `http://hôte:port`
pub fn address(base_url: &str) -> String {
    base_url.trim_start_matches("http://").to_string()
}

/// Texte d'une requête POST JSON avec des en-têtes supplémentaires
pub fn post_request(path: &str, headers: &[(&str, &str)], body: &str) -> String {
    let extra: String = headers
        .iter()
        .map(|(k, v)| format!("{k}: {v}\r\n"))
        .collect();
    format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n{extra}Content-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

/// Ouvre une connexion et écrit la requête sans lire la réponse
pub async fn open_and_send(base_url: &str, request: &str) -> TcpStream {
    let mut stream = TcpStream::connect(address(base_url)).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    stream
}

/// Lit jusqu'à la fermeture en gardant ce qui est reçu même si la lecture échoue
pub async fn read_all(stream: &mut TcpStream) -> Vec<u8> {
    let mut received = Vec::new();
    let mut buffer = [0u8; 4096];
    while let Ok(Ok(count)) = timeout(LIMIT, stream.read(&mut buffer)).await {
        if count == 0 {
            break;
        }
        received.extend_from_slice(&buffer[..count]);
    }
    received
}

/// Envoie la requête puis renvoie la réponse analysée
pub async fn exchange(base_url: &str, request: &str) -> Reply {
    let mut stream = open_and_send(base_url, request).await;
    parse_reply(&read_all(&mut stream).await)
}
