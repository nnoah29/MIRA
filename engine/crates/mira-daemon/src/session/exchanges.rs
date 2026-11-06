//! Registre des échanges en cours d'une connexion : un identifiant en cours ne se réutilise pas.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

/// Identifiants des échanges ouverts par le client et pas encore terminés.
#[derive(Debug, Clone, Default)]
pub struct Exchanges {
    ids: Arc<Mutex<HashSet<u64>>>,
}

impl Exchanges {
    /// Verrouille l'ensemble, même si une tâche a paniqué.
    fn ids(&self) -> MutexGuard<'_, HashSet<u64>> {
        self.ids
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Ouvre l'échange `id` ; faux s'il est déjà en cours.
    pub fn begin_call(&self, id: u64) -> bool {
        self.ids().insert(id)
    }

    /// Termine l'échange `id`.
    pub fn finish(&self, id: u64) {
        self.ids().remove(&id);
    }
}
