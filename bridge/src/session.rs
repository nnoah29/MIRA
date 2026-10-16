//! Moteur ouvert, partagé par tous les appels du pont pendant la vie du processus.

use std::sync::{Arc, Mutex, MutexGuard};

use mira_assembly::api::Engine;

static ENGINE: Mutex<Option<Arc<Engine>>> = Mutex::new(None);

/// Verrouille l'emplacement du moteur, même si un appel précédent a paniqué.
fn slot() -> MutexGuard<'static, Option<Arc<Engine>>> {
    ENGINE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Range le moteur ouvert et rend l'ancien s'il y en avait un.
pub fn install(engine: Arc<Engine>) -> Option<Arc<Engine>> {
    slot().replace(engine)
}

/// Rend une poignée sur le moteur ouvert, s'il y en a un.
pub fn current() -> Option<Arc<Engine>> {
    slot().clone()
}

/// Retire le moteur de l'emplacement et le rend à l'appelant.
pub fn take() -> Option<Arc<Engine>> {
    slot().take()
}
