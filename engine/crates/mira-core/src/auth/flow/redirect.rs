//! Écouteur de redirection en boucle locale : accepte la requête `/callback` puis se ferme.

use std::time::Duration;

use mira_vocabulary::error::codes::{authentication, invalid_input, network};
use mira_vocabulary::{ErrorCategory, MiraError};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

use super::callback::{AuthorizationCode, parse_callback};
use super::pending::PendingFlow;
use super::phase::FlowPhase;

const REQUEST_LIMIT: usize = 8192;
const READ_PATIENCE: Duration = Duration::from_secs(5);
const DONE_PAGE: &str = "<!doctype html><html lang=\"fr\"><head><meta charset=\"utf-8\"><title>MIRA</title></head><body><p>Autorisation reçue : vous pouvez revenir à MIRA.</p></body></html>";

/// Attend la redirection du navigateur, vérifie `state` et ferme l'écouteur
pub async fn wait_for_redirect(
    flow: &mut PendingFlow,
    limit: Duration,
) -> Result<AuthorizationCode, MiraError> {
    let listener = match (flow.phase, flow.listener.take()) {
        (FlowPhase::AwaitingRedirect, Some(listener)) => listener,
        _ => return Err(wrong_phase()),
    };
    let served = timeout(limit, serve_until_callback(&listener, &flow.state)).await;
    let result = served.unwrap_or_else(|_| {
        Err(MiraError::new(
            ErrorCategory::Network,
            network::TIMEOUT,
            "redirection non reçue à temps",
        ))
    });
    flow.phase = match &result {
        Ok(_) => FlowPhase::CodeReceived,
        Err(error) if error.code == authentication::OAUTH_CANCELLED.0 => FlowPhase::Cancelled,
        Err(_) => FlowPhase::Failed,
    };
    result
}

/// Erreur d'usage : le flux n'attend plus de redirection
pub(super) fn wrong_phase() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "flux d'autorisation dans un état incompatible",
    )
}

/// Sert les connexions jusqu'à la requête `/callback`, répond aux autres par 404
async fn serve_until_callback(
    listener: &TcpListener,
    state: &str,
) -> Result<AuthorizationCode, MiraError> {
    loop {
        let (mut stream, _) = listener.accept().await.map_err(|_| {
            MiraError::new(
                ErrorCategory::Network,
                network::CONNECTION_LOST,
                "écouteur local interrompu",
            )
        })?;
        let Some(target) = read_request_target(&mut stream).await else {
            continue;
        };
        let Some(query) = target.strip_prefix("/callback?") else {
            respond(&mut stream, "404 Not Found", "").await;
            continue;
        };
        let result = parse_callback(query, state);
        respond(&mut stream, "200 OK", DONE_PAGE).await;
        return result;
    }
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
