//! Contexte partagé par toutes les étapes de la synchronisation d'un compte.

use std::sync::Arc;

use mira_vocabulary::{Account, AccountId, EventKind, LogLevel, Provider, Timestamp};

use crate::events::EventSink;
use crate::ports::{Clock, IdGenerator};
use crate::storage::Database;

const MILLIS_PER_DAY: i64 = 86_400_000;

/// Dépendances d'une synchronisation : base, provider, compte, horloge, identifiants, événements.
#[derive(Debug, Clone)]
pub struct SyncContext {
    pub db: Arc<Database>,
    pub provider: Arc<dyn Provider>,
    pub account: Account,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    pub events: Arc<dyn EventSink>,
}

impl SyncContext {
    /// Identifiant du compte synchronisé.
    pub fn account_id(&self) -> &AccountId {
        &self.account.id
    }

    /// Fabrique un nouvel identifiant opaque.
    pub fn new_id(&self) -> String {
        self.ids.next_id()
    }

    /// Émet un événement ; un échec d'émission n'interrompt jamais la synchronisation.
    pub fn emit(&self, kind: EventKind) {
        if let Err(error) = self.events.emit(kind) {
            tracing::warn!(code = %error.code, "événement de synchronisation non émis");
        }
    }

    /// Instant correspondant au début d'une plage de `days` jours avant maintenant.
    pub fn window_start(&self, days: u32) -> Timestamp {
        self.clock
            .now()
            .plus_millis(-i64::from(days).saturating_mul(MILLIS_PER_DAY))
    }

    /// Journalise une décision de synchronisation et l'émet comme événement de diagnostic.
    pub fn diagnose(&self, level: LogLevel, target: &str, text: &str) {
        tracing::info!(target: "mira::sync", decision = target, "{text}");
        self.emit(EventKind::Diagnostic {
            level,
            target: target.to_string(),
            text: text.to_string(),
        });
    }
}
