//! Verrous par compte du rejeu : une passe en cours garde le sien, la suppression du compte l'attend.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mira_vocabulary::AccountId;

/// Un verrou asynchrone par compte, créé à la demande.
#[derive(Debug, Default)]
pub(crate) struct AccountGates {
    gates: Mutex<HashMap<AccountId, Arc<tokio::sync::Mutex<()>>>>,
}

impl AccountGates {
    /// Verrou du compte, partagé par toutes les demandes
    pub(crate) fn gate(&self, id: &AccountId) -> Arc<tokio::sync::Mutex<()>> {
        let mut table = self.gates.lock().unwrap_or_else(|p| p.into_inner());
        Arc::clone(table.entry(id.clone()).or_default())
    }
}
