//! Conversion entre une ligne de la table `identities` et une `Identity`.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{AccountId, EmailAddress, ErrorCategory, Identity, IdentityId, MiraError};

pub(super) const COLUMNS: &str = "id, account_id, address, display_name, signature, is_default";

/// Ligne brute de la table `identities`, dans l'ordre de `COLUMNS`.
pub(super) struct RawIdentity {
    id: String,
    account: String,
    address: String,
    display_name: String,
    signature: Option<String>,
    is_default: i64,
}

impl RawIdentity {
    /// Lit une ligne sélectionnée avec `COLUMNS`
    pub(super) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            account: row.get(1)?,
            address: row.get(2)?,
            display_name: row.get(3)?,
            signature: row.get(4)?,
            is_default: row.get(5)?,
        })
    }

    /// Reconstruit l'identité du domaine depuis la ligne
    pub(super) fn into_identity(self) -> Result<Identity, MiraError> {
        Ok(Identity {
            id: IdentityId::from_raw(self.id),
            account: AccountId::from_raw(self.account),
            address: EmailAddress::parse(&self.address)?,
            display_name: self.display_name,
            signature: self.signature,
            is_default: self.is_default != 0,
        })
    }
}

/// Erreur « identité inconnue »
pub(super) fn identity_not_found() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::IDENTITY,
        "identité inconnue",
    )
}
