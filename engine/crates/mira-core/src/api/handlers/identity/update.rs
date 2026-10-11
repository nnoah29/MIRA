//! Modification du nom et de la signature d'une identité.

use mira_vocabulary::{EventKind, IdentityId, MiraError};

use crate::accounts::{normalize_signature, require_active_account, require_identity};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Change le nom ou la signature ; une signature vide la supprime, une valeur absente ne change rien
pub(crate) fn handle_update_identity(
    engine: &Engine,
    id: &IdentityId,
    display_name: Option<String>,
    signature: Option<String>,
) -> Result<Response, MiraError> {
    let updated = engine.database.write(|tx| {
        let mut identity = require_identity(tx, id)?;
        require_active_account(tx, &identity.account)?;
        if let Some(name) = display_name {
            identity.display_name = name.trim().to_string();
        }
        if signature.is_some() {
            identity.signature = normalize_signature(signature);
        }
        tx.update_identity(&identity)?;
        Ok(identity)
    })?;
    engine.events.emit(EventKind::AccountUpdated {
        account: updated.account.clone(),
    })?;
    Ok(Response::Identity(updated))
}
