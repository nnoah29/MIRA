//! Ajout d'une identité d'envoi à un compte.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{EmailAddress, ErrorCategory, EventKind, Identity, IdentityId, MiraError};

use crate::accounts::{normalize_signature, require_active_account};
use crate::api::contract::{NewIdentity, Response};
use crate::api::engine::Engine;
use crate::events::EventSink;
use crate::storage::Tx;

/// Ajoute l'identité ; si elle devient celle par défaut, l'ancienne cesse de l'être
pub(crate) fn handle_add_identity(
    engine: &Engine,
    new: NewIdentity,
) -> Result<Response, MiraError> {
    let account = new.account;
    let identity = Identity {
        id: IdentityId::from_raw(engine.config.ids.next_id()),
        account: account.clone(),
        address: EmailAddress::parse(&new.address)?,
        display_name: new.display_name.trim().to_string(),
        signature: normalize_signature(new.signature),
        is_default: new.is_default,
    };
    engine.database.write(|tx| {
        require_active_account(tx, &account)?;
        reject_duplicate_address(tx, &identity)?;
        tx.insert_identity(&identity)
    })?;
    engine.events.emit(EventKind::AccountUpdated {
        account: account.clone(),
    })?;
    Ok(Response::Identity(identity))
}

/// Refuse une seconde identité avec la même adresse sur le même compte
fn reject_duplicate_address(tx: &Tx<'_>, identity: &Identity) -> Result<(), MiraError> {
    let known = tx.list_identities(&identity.account)?;
    if known.iter().any(|other| other.address == identity.address) {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "cette adresse est déjà une identité du compte",
        ));
    }
    Ok(())
}
