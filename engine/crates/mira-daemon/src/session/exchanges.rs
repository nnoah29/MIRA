//! Registre des échanges en cours d'une connexion : un identifiant en cours ne se réutilise pas.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use super::stream::Stream;

/// Un échange ouvert par le client : un appel, ou un flux avec son état.
#[derive(Debug)]
pub(super) enum Entry {
    Call,
    Stream(Stream),
}

/// Échanges ouverts par le client et pas encore terminés.
#[derive(Debug, Clone, Default)]
pub struct Exchanges {
    entries: Arc<Mutex<HashMap<u64, Entry>>>,
}

impl Exchanges {
    /// Verrouille le registre, même si une tâche a paniqué.
    pub(super) fn entries(&self) -> MutexGuard<'_, HashMap<u64, Entry>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Insère l'échange `id` ; faux s'il est déjà en cours.
    fn begin(&self, id: u64, entry: Entry) -> bool {
        let mut entries = self.entries();
        let fresh = !entries.contains_key(&id);
        if fresh {
            entries.insert(id, entry);
        }
        fresh
    }

    /// Ouvre un appel `id` ; faux s'il est déjà en cours.
    pub fn begin_call(&self, id: u64) -> bool {
        self.begin(id, Entry::Call)
    }

    /// Ouvre un flux `id` ; faux si l'identifiant est déjà en cours.
    pub fn begin_stream(&self, id: u64, stream: Stream) -> bool {
        self.begin(id, Entry::Stream(stream))
    }

    /// Termine l'échange `id`.
    pub fn finish(&self, id: u64) {
        self.entries().remove(&id);
    }
}
