//! Statut d'une règle calculé à la lecture : valide, ou invalide avec la raison.

use mira_vocabulary::Rule;
use serde::{Deserialize, Serialize};

use super::mailboxes::check_world;
use super::world::RuleWorld;

/// Statut d'une règle ; une règle invalide n'est jamais exécutée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleStatus {
    Valid,
    Invalid { reason: String },
}

/// Calcule le statut d'une règle d'après les comptes et boîtes qui existent maintenant.
pub fn rule_status(rule: &Rule, world: &dyn RuleWorld) -> RuleStatus {
    match check_world(rule, world) {
        Ok(()) => RuleStatus::Valid,
        Err(error) => RuleStatus::Invalid {
            reason: error.message,
        },
    }
}
