//! Boucle d'acceptation des connexions SMTP.

use tokio::net::TcpListener;
use tokio::task::JoinSet;

use super::session::Session;
use super::state::SmtpShared;
use crate::credentials::Credentials;

/// Accepte les connexions et lance une session par client jusqu'à l'arrêt
pub(crate) async fn accept_loop(
    listener: TcpListener,
    shared: SmtpShared,
    credentials: Credentials,
) {
    let mut sessions = JoinSet::new();
    while let Ok((stream, _)) = listener.accept().await {
        while sessions.try_join_next().is_some() {}
        let session = Session::new(stream, shared.clone(), credentials.clone());
        sessions.spawn(session.run());
    }
}
