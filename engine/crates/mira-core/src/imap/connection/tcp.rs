//! Ouverture de la connexion TCP vers le serveur.

use std::time::Duration;

use mira_vocabulary::{MiraError, ServerEndpoint};
use tokio::net::TcpStream;

use crate::imap::errors::network::{timeout, unreachable};

/// Ouvre la connexion TCP dans le délai donné.
pub(crate) async fn connect_tcp(
    endpoint: &ServerEndpoint,
    wait: Duration,
) -> Result<TcpStream, MiraError> {
    let target = (endpoint.host.as_str(), endpoint.port);
    match tokio::time::timeout(wait, TcpStream::connect(target)).await {
        Err(_) => Err(timeout()),
        Ok(Err(_)) => Err(unreachable(&endpoint.host, endpoint.port)),
        Ok(Ok(stream)) => {
            let _ = stream.set_nodelay(true);
            Ok(stream)
        }
    }
}
