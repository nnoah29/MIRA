//! Identité d'envoi d'un compte.

use serde::{Deserialize, Serialize};

use crate::address::EmailAddress;
use crate::ids::kinds::{AccountId, IdentityId};

/// Adresse et signature sous lesquelles l'utilisateur écrit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub id: IdentityId,
    pub account: AccountId,
    pub address: EmailAddress,
    pub display_name: String,
    pub signature: Option<String>,
    pub is_default: bool,
}
