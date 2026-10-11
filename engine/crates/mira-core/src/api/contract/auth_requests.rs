//! Requêtes du domaine authentification (12.4, 7.7, 15.2).

use mira_vocabulary::AccountId;
use serde::{Deserialize, Serialize};

use super::secret_input::SecretInput;

/// Opérations d'authentification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AuthRequest {
    StoreAppPassword {
        account: AccountId,
        password: SecretInput,
    },
}
