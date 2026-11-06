//! Boucle d'acceptation : une tâche de session par connexion jusqu'au signal d'arrêt.

use std::sync::Arc;
use std::time::Duration;

use super::{ServerContext, wait_closing};
use crate::session;
use crate::transport::{Accepted, Listener};

/// Accepte les connexions de cet écouteur jusqu'à l'arrêt du serveur.
pub async fn serve<L: Listener + 'static>(listener: L, ctx: Arc<ServerContext>) {
    let mut closing = ctx.closing_receiver();
    loop {
        tokio::select! {
            accepted = listener.accept() => start_session(accepted, &ctx).await,
            _ = wait_closing(&mut closing) => return,
        }
    }
}

/// Lance la session d'une connexion acceptée ; une erreur d'acceptation fait patienter un instant.
async fn start_session(accepted: std::io::Result<Accepted>, ctx: &Arc<ServerContext>) {
    match accepted {
        Ok(accepted) => {
            tokio::spawn(session::run(accepted, ctx.clone()));
        }
        Err(_) => tokio::time::sleep(Duration::from_millis(50)).await,
    }
}
