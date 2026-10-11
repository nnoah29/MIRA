//! Relecture des cibles serveur mémorisées d'une action.

use mira_vocabulary::{ActionId, MiraError};

use super::model::ServerTargets;
use crate::actions::errors::not_found::action_not_found;
use crate::storage::Tx;
use crate::storage::json_columns::from_json;

/// Relit les cibles serveur mémorisées à la soumission de l'action
pub fn load_server_targets(tx: &Tx<'_>, id: &ActionId) -> Result<ServerTargets, MiraError> {
    let payload = tx
        .get_action_payload(id)?
        .ok_or_else(|| action_not_found(id))?;
    if payload.server_targets_json == "[]" {
        return Ok(ServerTargets::default());
    }
    from_json(&payload.server_targets_json)
}
