//! Suivi des changements d'état annoncé par le serveur (icône, tests).

use mira_protocol::HostState;
use tokio::sync::watch;

use super::ServerContext;

impl ServerContext {
    /// Récepteur des changements de l'état annoncé aux clients.
    pub fn state_receiver(&self) -> watch::Receiver<HostState> {
        self.state.subscribe()
    }
}
