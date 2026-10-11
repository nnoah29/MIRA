//! Recherche d'un compte ou d'une identité existants, avec les erreurs du contrat.

use mira_vocabulary::error::codes::{incompatible_state, not_found};
use mira_vocabulary::{
    Account, AccountId, ErrorCategory, ErrorContext, Identity, IdentityId, MiraError,
};

use crate::storage::Tx;

/// Relit un compte ou échoue avec `not_found.account`
pub fn require_account(tx: &Tx<'_>, id: &AccountId) -> Result<Account, MiraError> {
    tx.get_account(id)?.ok_or_else(|| {
        MiraError::new(
            ErrorCategory::NotFound,
            not_found::ACCOUNT,
            "compte inconnu",
        )
        .with_context(context_of(id))
    })
}

/// Relit un compte qui n'est pas en cours de suppression
pub fn require_active_account(tx: &Tx<'_>, id: &AccountId) -> Result<Account, MiraError> {
    let account = require_account(tx, id)?;
    if tx.account_is_deleting(id)? {
        return Err(MiraError::new(
            ErrorCategory::IncompatibleState,
            incompatible_state::ACCOUNT_DELETING,
            "le compte est en cours de suppression",
        )
        .with_context(context_of(id)));
    }
    Ok(account)
}

/// Relit une identité ou échoue avec `not_found.identity`
pub fn require_identity(tx: &Tx<'_>, id: &IdentityId) -> Result<Identity, MiraError> {
    tx.get_identity(id)?.ok_or_else(|| {
        MiraError::new(
            ErrorCategory::NotFound,
            not_found::IDENTITY,
            "identité inconnue",
        )
    })
}

/// Contexte d'erreur qui désigne le compte visé
fn context_of(id: &AccountId) -> ErrorContext {
    ErrorContext {
        account: Some(id.clone()),
        ..ErrorContext::default()
    }
}
