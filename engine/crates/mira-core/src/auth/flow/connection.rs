//! Une connexion de l'écouteur de redirection : lecture de la requête et réponse minimale.

use std::time::Duration;

use mira_vocabulary::MiraError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::callback::{AuthorizationCode, parse_callback};
use super::state_check::is_state_mismatch;

const REQUEST_LIMIT: usize = 8192;
const READ_PATIENCE: Duration = Duration::from_secs(5);
const CONNECTION_PATIENCE: Duration = Duration::from_secs(10);
const DONE_PAGE: &str = "<!doctype html><html lang=\"fr\"><head><meta charset=\"utf-8\"><title>MIRA</title></head><body><p>Autorisation reçue : vous pouvez revenir à MIRA.</p></body></html>";

/// Traite une connexion ; ne rend un résultat que pour une redirection portant le bon `state`
pub(super) async fn serve_connection(
    stream: TcpStream,
    state: String,
) -> Option<Result<AuthorizationCode, MiraError>> {
    timeout(CONNECTION_PATIENCE, answer_request(stream, &state))
        .await
        .ok()
        .flatten()
}

/// Lit la requête puis répond : 404 hors `/callback`, 400 si `state` est faux, 200 sinon
async fn answer_request(
    mut stream: TcpStream,
    state: &str,
) -> Option<Result<AuthorizationCode, MiraError>> {
    let target = read_request_target(&mut stream).await?;
    let Some(query) = target.strip_prefix("/callback?") else {
        respond(&mut stream, "404 Not Found", "").await;
        return None;
    };
    let result = parse_callback(query, state);
    if matches!(&result, Err(error) if is_state_mismatch(error)) {
        respond(&mut stream, "400 Bad Request", "").await;
        return None;
    }
    respond(&mut stream, "200 OK", DONE_PAGE).await;
    Some(result)
}

/// Lit la ligne de requête d'un `GET` et renvoie sa cible, ou rien si la requête est inutilisable
async fn read_request_target(stream: &mut TcpStream) -> Option<String> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    while !buffer.windows(4).any(|window| window == b"\r\n\r\n") && buffer.len() < REQUEST_LIMIT {
        let read = timeout(READ_PATIENCE, stream.read(&mut chunk))
            .await
            .ok()?
            .ok()?;
        if read == 0 {
            return None;
        }
        buffer.extend_from_slice(chunk.get(..read)?);
    }
    let text = String::from_utf8_lossy(&buffer).into_owned();
    let mut parts = text.lines().next()?.split_whitespace();
    (parts.next()? == "GET").then(|| parts.next().map(str::to_string))?
}

/// Écrit une réponse HTTP minimale (sans JavaScript) puis ferme la connexion
async fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let reply = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(reply.as_bytes()).await;
    let _ = stream.shutdown().await;
}
