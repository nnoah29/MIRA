//! Requêtes du domaine « file d'actions » : lister, relancer, annuler (12.4).

use mira_vocabulary::{AccountId, ActionId, ActionState};
use serde::{Deserialize, Serialize};

/// Opérations sur la file d'actions locale d'un compte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum QueueRequest {
    ListActions {
        account: AccountId,
        states: Vec<ActionState>,
    },
    RetryAction {
        action: ActionId,
    },
    CancelAction {
        action: ActionId,
    },
}
