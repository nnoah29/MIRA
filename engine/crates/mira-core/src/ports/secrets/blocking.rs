//! Exécution d'un appel bloquant (trousseau D-Bus) sans bloquer l'exécuteur asynchrone.

use tokio::runtime::{Handle, RuntimeFlavor};

/// Exécute un appel bloquant ; sur un exécuteur multi-fils, les autres tâches quittent ce fil
pub fn run_blocking_call<T>(operation: impl FnOnce() -> T) -> T {
    match Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(operation)
        }
        _ => operation(),
    }
}
