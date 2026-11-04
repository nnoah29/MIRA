//! L'hôte : l'emplacement du moteur ouvert et son accès partagé.

use std::sync::{Arc, Mutex, MutexGuard};

use mira_assembly::api::Engine;

/// Possède au plus un moteur ouvert, partagé par tous les appels faits sur cet hôte.
#[derive(Debug, Default)]
pub struct Host {
    slot: Mutex<Option<Arc<Engine>>>,
}

impl Host {
    /// Hôte sans moteur ouvert.
    pub const fn new() -> Self {
        Self {
            slot: Mutex::new(None),
        }
    }

    /// Verrouille l'emplacement du moteur, même si un appel précédent a paniqué.
    fn slot(&self) -> MutexGuard<'_, Option<Arc<Engine>>> {
        self.slot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Rend une poignée sur le moteur ouvert, s'il y en a un.
    pub fn engine(&self) -> Option<Arc<Engine>> {
        self.slot().clone()
    }

    /// Range le moteur ouvert et rend l'ancien s'il y en avait un.
    pub(crate) fn install(&self, engine: Arc<Engine>) -> Option<Arc<Engine>> {
        self.slot().replace(engine)
    }

    /// Retire le moteur de l'emplacement et le rend à l'appelant.
    pub(crate) fn take(&self) -> Option<Arc<Engine>> {
        self.slot().take()
    }
}
