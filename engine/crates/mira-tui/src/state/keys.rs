//! Fabrication des clés d'idempotence de la session.

use mira_assembly::mira_vocabulary::IdempotencyKey;

use super::AppState;
use crate::commands::idempotency::next_key;

impl AppState {
    /// Clé neuve `tui-<nonce>-<étiquette>-<n>` : le nonce sépare les sessions, le compteur les appels.
    pub fn fresh_key(&mut self, label: &str) -> IdempotencyKey {
        let prefix = format!("{}-{label}", self.key_prefix);
        next_key(&mut self.key_counter, &prefix)
    }
}
