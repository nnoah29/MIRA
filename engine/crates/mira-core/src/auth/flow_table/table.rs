//! Table des flux OAuth en cours, protégée par un verrou jamais tenu pendant une attente.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use super::entry::FlowEntry;

const DEFAULT_FLOW_TTL: Duration = Duration::from_secs(300);

/// Flux en cours, par identifiant de flux ; un flux qui attend trop longtemps est oublié.
#[derive(Debug)]
pub struct OAuthFlows {
    entries: Mutex<BTreeMap<String, FlowEntry>>,
    pub(super) ttl: Duration,
}

impl Default for OAuthFlows {
    /// Crée une table vide de durée de vie 300 s
    fn default() -> Self {
        Self::new()
    }
}

impl OAuthFlows {
    /// Crée une table vide de durée de vie 300 s
    pub fn new() -> Self {
        Self::with_ttl(DEFAULT_FLOW_TTL)
    }

    /// Crée une table vide où un flux en attente expire après `ttl`
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
            ttl,
        }
    }

    /// Prend le verrou de la table en ignorant un éventuel empoisonnement
    pub(super) fn entries(&self) -> MutexGuard<'_, BTreeMap<String, FlowEntry>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Enregistre un flux sous son identifiant et oublie les flux expirés
    pub fn register(&self, flow: String, entry: FlowEntry) {
        let mut entries = self.entries();
        self.purge_expired(&mut entries);
        entries.insert(flow, entry);
    }

    /// Oublie un flux, terminé ou non
    pub fn remove(&self, flow: &str) {
        self.entries().remove(flow);
    }
}
