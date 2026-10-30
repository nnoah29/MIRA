//! Poignée du faux serveur IMAP.

use std::sync::Arc;

use tokio::task::JoinHandle;

use super::tracker::Tracker;
use crate::imap::store::FakeMailStore;

/// Faux serveur IMAP en écoute sur 127.0.0.1 ; s'arrête quand la poignée est détruite.
#[derive(Debug)]
pub struct FakeImapServer {
    pub(super) port: u16,
    pub(super) store: FakeMailStore,
    pub(super) tracker: Arc<Tracker>,
    pub(super) listener: JoinHandle<()>,
}

impl FakeImapServer {
    /// Port TCP éphémère d'écoute
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Adresse `127.0.0.1:<port>`
    pub fn address(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// Magasin de messages du serveur, pour agir « depuis un autre appareil »
    pub fn state(&self) -> FakeMailStore {
        self.store.clone()
    }

    /// Nombre total de connexions acceptées depuis le démarrage
    pub fn connection_count(&self) -> usize {
        self.tracker.total()
    }

    /// Nombre de connexions actuellement ouvertes
    pub fn active_connections(&self) -> usize {
        self.tracker.active()
    }

    /// Nombre de sessions IDLE établies : abonnées au magasin, donc sûres d'annoncer toute livraison suivante
    pub fn idle_sessions(&self) -> usize {
        self.store.shared.version.receiver_count()
    }
}
