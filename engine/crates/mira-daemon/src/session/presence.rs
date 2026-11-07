//! Présence d'une connexion : enregistrée sous une clé propre à la connexion, retirée quand elle se ferme.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use mira_host::Host;

static NEXT_CONNECTION: AtomicU64 = AtomicU64::new(1);

/// Entrée de présence d'une connexion ; deux connexions d'un même client comptent séparément.
#[derive(Debug)]
pub struct PresenceKey {
    host: Arc<Host>,
    key: String,
}

impl PresenceKey {
    /// Clé neuve pour une connexion de cet hôte.
    pub fn new(host: Arc<Host>) -> Self {
        let number = NEXT_CONNECTION.fetch_add(1, Ordering::Relaxed);
        Self {
            host,
            key: format!("connection-{number}"),
        }
    }

    /// Déclare le focus de la connexion.
    pub fn set(&self, focused: bool) {
        self.host.presence().set(&self.key, focused);
    }
}

impl Drop for PresenceKey {
    /// Retire la présence de la connexion fermée.
    fn drop(&mut self) {
        self.host.presence().remove(&self.key);
    }
}
