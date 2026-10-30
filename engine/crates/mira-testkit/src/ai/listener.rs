//! Boucle d'acceptation et service d'une connexion : lecture, réponse, détection du client parti.

use std::io;
use std::time::Duration;

use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio::time::timeout;

use super::plain::write_plain;
use super::request::read_request;
use super::respond::{finish, respond};
use super::state::AiShared;

const READ_LIMIT: Duration = Duration::from_secs(10);

/// Accepte les connexions et sert chacune dans sa tâche jusqu'à l'arrêt
pub(super) async fn accept_loop(listener: TcpListener, shared: AiShared) {
    let mut connections = JoinSet::new();
    while let Ok((stream, _)) = listener.accept().await {
        while connections.try_join_next().is_some() {}
        connections.spawn(serve(stream, shared.clone()));
    }
}

/// Lit la requête, joue le comportement et compte la fermeture si le client part avant la fin
async fn serve(stream: TcpStream, shared: AiShared) {
    let (read_half, mut writer) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let Ok(Ok(request)) = timeout(READ_LIMIT, read_request(&mut reader)).await else {
        let _ = write_plain(&mut writer, 400, &[], r#"{"error":"invalid_request"}"#).await;
        return;
    };
    let behavior = shared.with(|state| {
        state.requests.push(request);
        state.next_behavior()
    });
    let outcome = tokio::select! {
        result = respond(&mut writer, behavior) => result,
        () = wait_for_eof(reader.into_inner()) => Err(io::ErrorKind::ConnectionAborted.into()),
    };
    match outcome {
        Ok(()) => finish(&mut writer).await,
        Err(_) => shared.with(|state| state.closed_by_client += 1),
    }
}

/// Retourne quand le client a fermé sa connexion ou que la lecture échoue
async fn wait_for_eof(mut read_half: OwnedReadHalf) {
    let mut buffer = [0u8; 1024];
    while let Ok(count) = read_half.read(&mut buffer).await {
        if count == 0 {
            return;
        }
    }
}
