//! Verrous des rafraîchissements : un par connexion, pour qu'un seul renouvellement se fasse à la fois.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mira_vocabulary::AiConnectionId;

/// Verrous asynchrones des connexions, créés à la demande.
#[derive(Debug, Default)]
pub struct AiSessionLocks {
    locks: Mutex<HashMap<AiConnectionId, Arc<tokio::sync::Mutex<()>>>>,
}

impl AiSessionLocks {
    /// Verrou de la connexion, créé au premier appel
    pub(crate) fn of(&self, id: &AiConnectionId) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = self
            .locks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::clone(locks.entry(id.clone()).or_default())
    }
}
