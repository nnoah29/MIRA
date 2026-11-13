//! Table des connexions par navigateur en cours, protégée par un verrou jamais tenu pendant une attente.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use mira_vocabulary::AiConnectionId;
use tokio::sync::Notify;

use crate::auth::PendingFlow;

const FLOW_TTL: Duration = Duration::from_secs(300);

/// Ce que le flux rend : une clé d'API reçue une fois, ou une session de jetons renouvelables (avec l'identifiant d'hôte annoncé).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyLoginKind {
    Key,
    Session { host_id: String },
}

/// Flux en cours : le flux lui-même tant qu'il attend, son signal d'annulation et la connexion visée.
#[derive(Debug)]
pub struct KeyLoginEntry {
    pub(super) pending: Option<PendingFlow>,
    pub(super) cancel: Arc<Notify>,
    pub(super) connection: AiConnectionId,
    pub(super) kind: KeyLoginKind,
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

    /// Enregistre un flux qui rendra une clé d'API
    pub fn register(&self, flow: String, connection: AiConnectionId, pending: PendingFlow) {
        self.register_kind(flow, connection, pending, KeyLoginKind::Key);
    }

    /// Enregistre un flux qui rendra une session ; l'identifiant d'hôte est celui annoncé au service
    pub fn register_session(
        &self,
        flow: String,
        connection: AiConnectionId,
        pending: PendingFlow,
        host_id: String,
    ) {
        let kind = KeyLoginKind::Session { host_id };
        self.register_kind(flow, connection, pending, kind);
    }

    /// Enregistre un flux pour la connexion et oublie les flux expirés
    fn register_kind(
        &self,
        flow: String,
        connection: AiConnectionId,
        pending: PendingFlow,
        kind: KeyLoginKind,
    ) {
        let entry = KeyLoginEntry {
            pending: Some(pending),
            cancel: Arc::new(Notify::new()),
            connection,
            kind,
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
