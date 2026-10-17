//! Requêtes du domaine identités d'envoi (12.4, 7.5).

use mira_vocabulary::{AccountId, IdentityId};
use serde::{Deserialize, Serialize};

/// Opérations sur les identités d'envoi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum IdentityRequest {
    ListIdentities {
        account: AccountId,
    },
    AddIdentity(NewIdentity),
    UpdateIdentity {
        identity: IdentityId,
        display_name: Option<String>,
        signature: Option<String>,
    },
    SetDefaultIdentity {
        identity: IdentityId,
    },
}

/// Identité à ajouter à un compte ; une signature vide signifie « aucune signature ».
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewIdentity {
    pub account: AccountId,
    pub address: String,
    pub display_name: String,
    pub signature: Option<String>,
    pub is_default: bool,
}
