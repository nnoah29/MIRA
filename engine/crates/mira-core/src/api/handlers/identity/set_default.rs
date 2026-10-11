//! Désignation de l'identité par défaut d'un compte (une seule à la fois).

use mira_vocabulary::{EventKind, Identity, IdentityId, MiraError};

use crate::accounts::{require_active_account, require_identity};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Fait de cette identité la seule identité par défaut de son compte
pub(crate) fn handle_set_default_identity(
    engine: &Engine,
    id: &IdentityId,
) -> Result<Response, MiraError> {
    let chosen = engine.database.write(|tx| {
        let identity = require_identity(tx, id)?;
        require_active_account(tx, &identity.account)?;
        tx.set_default_identity(&identity.account, id)?;
        Ok(Identity {
            is_default: true,
            ..identity
        })
    })?;
    engine.events.emit(EventKind::AccountUpdated {
        account: chosen.account.clone(),
    })?;
    Ok(Response::Identity(chosen))
}
