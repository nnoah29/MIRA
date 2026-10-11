//! Requêtes du domaine authentification (12.4, 7.7, 15.2).

use mira_vocabulary::{AccountId, ProviderId};
use serde::{Deserialize, Serialize};

use super::secret_input::SecretInput;

/// Opérations d'authentification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AuthRequest {
    #[serde(rename = "start_oauth")]
    StartOAuth {
        address: String,
        provider: Option<ProviderId>,
    },
    #[serde(rename = "complete_oauth")]
    CompleteOAuth {
        flow: String,
    },
    #[serde(rename = "cancel_oauth")]
    CancelOAuth {
        flow: String,
    },
    StoreAppPassword {
        account: AccountId,
        password: SecretInput,
    },
    Reauthenticate {
        account: AccountId,
    },
}
