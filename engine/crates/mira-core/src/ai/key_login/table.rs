//! Table des connexions par navigateur en cours, protégée par un verrou jamais tenu pendant une attente.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use mira_vocabulary::AiConnectionId;
use tokio::sync::Notify;

use crate::auth::PendingFlow;

const FLOW_TTL: Duration = Duration::from_secs(300);

/// Flux en cours : le flux lui-même tant qu'il attend, son signal d'annulation et la connexion visée.
#[derive(Debug)]
pub struct KeyLoginEntry {
    pub(super) pending: Option<PendingFlow>,
    pub(super) cancel: Arc<Notify>,
    pub(super) connection: AiConnectionId,
    pub(super) created: Instant,
}

/// Connexions par navigateur en cours, par identifiant de flux ; un flux trop vieux est oublié.
#[derive(Debug, Default)]
pub struct AiKeyLogins {
    entries: Mutex<BTreeMap<String, KeyLoginEntry>>,
}

impl AiKeyLogins {
    /// Prend le verrou de la table en ignorant un éventuel empoisonnement
    pub(super) fn entries(&self) -> MutexGuard<'_, BTreeMap<String, KeyLoginEntry>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Enregistre un flux pour la connexion et oublie les flux expirés
    pub fn register(&self, flow: String, connection: AiConnectionId, pending: PendingFlow) {
        let entry = KeyLoginEntry {
            pending: Some(pending),
            cancel: Arc::new(Notify::new()),
            connection,
            created: Instant::now(),
        };
        let mut entries = self.entries();
        entries.retain(|_, other| other.created.elapsed() < FLOW_TTL);
        entries.insert(flow, entry);
    }

    /// Oublie un flux, terminé ou non
    pub fn remove(&self, flow: &str) {
        self.entries().remove(flow);
    }
}
