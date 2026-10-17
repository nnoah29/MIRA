//! Boucle d'acceptation et traitement d'une connexion HTTP.

use std::time::Duration;

use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio::time::timeout;

use super::context::OAuthContext;
use super::http::request::read_request;
use super::http::response::HttpResponse;
use super::router::route;

const READ_LIMIT: Duration = Duration::from_secs(10);

/// Accepte les connexions et sert chacune dans sa tâche jusqu'à l'arrêt
pub(crate) async fn accept_loop(listener: TcpListener, context: OAuthContext) {
    let mut connections = JoinSet::new();
    while let Ok((stream, _)) = listener.accept().await {
        while connections.try_join_next().is_some() {}
        connections.spawn(serve(stream, context.clone()));
    }
}

/// Lit une requête, répond puis ferme la connexion
async fn serve(stream: TcpStream, context: OAuthContext) {
    let mut reader = BufReader::new(stream);
    let response = match timeout(READ_LIMIT, read_request(&mut reader)).await {
        Ok(Ok(request)) => route(&request, &context),
        _ => HttpResponse::error(400, "invalid_request"),
    };
    if response.write_to(reader.get_mut()).await.is_ok() {
        let _ = reader.get_mut().shutdown().await;
    }
}
