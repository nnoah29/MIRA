//! Arrêt et coupures du faux serveur.

use super::handle::FakeImapServer;

impl FakeImapServer {
    /// Coupe brutalement toutes les connexions ouvertes (coupure réseau simulée)
    pub fn drop_all_connections(&self) {
        self.tracker.abort_all();
    }

    /// Arrête l'écoute et coupe toutes les connexions
    pub fn shutdown(&self) {
        self.listener.abort();
        self.tracker.abort_all();
    }
}

impl Drop for FakeImapServer {
    /// L'arrêt de la poignée arrête le serveur
    fn drop(&mut self) {
        self.shutdown();
    }
}
