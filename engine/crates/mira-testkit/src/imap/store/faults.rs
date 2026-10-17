//! Journal des commandes et pannes injectables.

use std::sync::{Mutex, MutexGuard};

/// Pannes en attente et journal des commandes reçues.
#[derive(Debug, Default)]
pub(crate) struct Faults {
    failures: Mutex<Vec<(String, String)>>,
    log: Mutex<Vec<String>>,
}

/// Verrouille en ignorant un éventuel empoisonnement
fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Faults {
    /// Programme une réponse d'erreur pour la prochaine commande de ce nom
    pub(crate) fn push_failure(&self, command: &str, response: &str) {
        let key = command.split_whitespace().collect::<Vec<_>>().join(" ");
        locked(&self.failures).push((key.to_ascii_uppercase(), response.to_string()));
    }

    /// Retire la panne programmée pour l'une des clés données
    pub(crate) fn take_failure(&self, keys: &[String]) -> Option<String> {
        let mut failures = locked(&self.failures);
        let index = failures.iter().position(|(key, _)| keys.contains(key))?;
        Some(failures.remove(index).1)
    }

    /// Ajoute une commande au journal
    pub(crate) fn record(&self, line: String) {
        locked(&self.log).push(line);
    }

    /// Copie du journal des commandes
    pub(crate) fn snapshot(&self) -> Vec<String> {
        locked(&self.log).clone()
    }
}
