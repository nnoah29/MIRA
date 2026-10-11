//! Choix de l'identité d'envoi d'un brouillon : celle demandée, sinon l'identité par défaut du compte.

use mira_vocabulary::{AccountId, Identity, IdentityId, MiraError};

use crate::actions::errors::not_found::{identity_not_found, no_identity_for_account};
use crate::storage::Tx;

/// Identité demandée (elle doit appartenir au compte) ou identité par défaut du compte
pub(crate) fn resolve_identity(
    tx: &Tx<'_>,
    account: &AccountId,
    wanted: Option<&IdentityId>,
) -> Result<Identity, MiraError> {
    match wanted {
        Some(id) => match tx.get_identity(id)? {
            Some(identity) if &identity.account == account => Ok(identity),
            _ => Err(identity_not_found(id)),
        },
        None => tx
            .list_identities(account)?
            .into_iter()
            .next()
            .ok_or_else(|| no_identity_for_account(account)),
    }
}
