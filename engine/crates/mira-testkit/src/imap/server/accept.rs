//! Boucle d'acceptation des connexions.

use std::sync::Arc;

use tokio::net::TcpListener;

use super::context::ServerContext;
use super::tracker::Tracker;
use crate::imap::session::run::run_session;

/// Accepte les connexions et lance une session par connexion
pub(crate) async fn accept_loop(
    listener: TcpListener,
    context: Arc<ServerContext>,
    tracker: Arc<Tracker>,
) {
    while let Ok((stream, _)) = listener.accept().await {
        let session = run_session(stream, Arc::clone(&context));
        tracker.register(tokio::spawn(session));
    }
}
