//! Session SMTP d'un client : état de la transaction et boucle de commandes.

mod dispatch;
mod lines;
mod transaction;

use tokio::io::BufReader;
use tokio::net::TcpStream;

use super::state::SmtpShared;
use crate::credentials::Credentials;
use dispatch::dispatch;
pub(crate) use transaction::Transaction;

/// Suite à donner à la connexion après une commande.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flow {
    Continue,
    Close,
}

/// Connexion cliente en cours de dialogue.
pub(crate) struct Session {
    pub(crate) stream: BufReader<TcpStream>,
    pub(crate) shared: SmtpShared,
    pub(crate) credentials: Credentials,
    pub(crate) transaction: Transaction,
}

impl Session {
    /// Crée la session d'un client fraîchement connecté
    pub(crate) fn new(stream: TcpStream, shared: SmtpShared, credentials: Credentials) -> Self {
        Self {
            stream: BufReader::new(stream),
            shared,
            credentials,
            transaction: Transaction::default(),
        }
    }

    /// Joue la bannière puis traite les commandes jusqu'à la fermeture
    pub(crate) async fn run(mut self) {
        if self
            .reply("220 127.0.0.1 MIRA testkit ESMTP")
            .await
            .is_err()
        {
            return;
        }
        while let Ok(line) = self.read_line().await {
            if !matches!(dispatch(&mut self, &line).await, Ok(Flow::Continue)) {
                return;
            }
        }
    }
}
