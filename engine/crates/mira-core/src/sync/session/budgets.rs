//! Budgets de connexions de tous les comptes d'un moteur : un seul par compte, partagé par tous ses usages.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::budget::ConnectionBudget;

/// Table des budgets, créés au premier usage d'un compte avec la limite de son provider.
#[derive(Debug, Clone, Default)]
pub struct ConnectionBudgets {
    by_key: Arc<Mutex<HashMap<String, ConnectionBudget>>>,
}

impl ConnectionBudgets {
    /// Budget associé à la clé (identifiant du compte, ou adresse d'un compte pas encore ajouté).
    pub fn budget_for(&self, key: &str, limit: u32) -> ConnectionBudget {
        let mut table = self.by_key.lock().unwrap_or_else(|p| p.into_inner());
        table
            .entry(key.to_string())
            .or_insert_with(|| ConnectionBudget::new(limit))
            .clone()
    }

    /// Oublie le budget d'un compte supprimé.
    pub fn forget(&self, key: &str) {
        let mut table = self.by_key.lock().unwrap_or_else(|p| p.into_inner());
        table.remove(key);
    }
}
