//! Présence des clients : qui regarde MIRA en ce moment. Mécanisme d'hôte, sans lien avec les notifications.

use std::sync::{Mutex, MutexGuard};

/// État de focus de chaque client attaché ; vide, personne ne regarde.
#[derive(Debug, Default)]
pub struct Presence {
    clients: Mutex<Vec<(String, bool)>>,
}

impl Presence {
    /// Présence sans aucun client.
    pub const fn new() -> Self {
        Self {
            clients: Mutex::new(Vec::new()),
        }
    }

    /// Verrouille la table, même si un appel précédent a paniqué.
    fn table(&self) -> MutexGuard<'_, Vec<(String, bool)>> {
        self.clients
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Enregistre l'état de focus de ce client, en remplaçant le précédent.
    pub fn set(&self, client: &str, focused: bool) {
        let mut table = self.table();
        match table.iter_mut().find(|(name, _)| name == client) {
            Some(entry) => entry.1 = focused,
            None => table.push((client.to_string(), focused)),
        }
    }

    /// Oublie ce client ; un client inconnu est sans effet.
    pub fn remove(&self, client: &str) {
        self.table().retain(|(name, _)| name != client);
    }

    /// Vrai si au moins un client est au premier plan.
    pub fn is_active(&self) -> bool {
        self.table().iter().any(|(_, focused)| *focused)
    }
}
