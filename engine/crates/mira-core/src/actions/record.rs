//! Lecture et écriture de l'enregistrement d'annulation d'une action (colonne `undo_json`).

use mira_vocabulary::{ActionId, MiraError};

use super::errors::not_found::action_not_found;
use super::model::UndoRecord;
use crate::storage::Tx;
use crate::storage::json_columns::{from_json, to_json};
use crate::storage::repositories::actions::ActionPayload;

/// Relit l'enregistrement d'annulation d'une action, ou rien si elle n'en a pas
pub(crate) fn load_record(tx: &Tx<'_>, id: &ActionId) -> Result<Option<UndoRecord>, MiraError> {
    let payload = tx
        .get_action_payload(id)?
        .ok_or_else(|| action_not_found(id))?;
    payload.undo_json.as_deref().map(from_json).transpose()
}

/// Remplace l'enregistrement d'annulation en gardant les cibles serveur
pub(crate) fn save_record(
    tx: &Tx<'_>,
    id: &ActionId,
    record: &UndoRecord,
) -> Result<(), MiraError> {
    let payload = tx
        .get_action_payload(id)?
        .ok_or_else(|| action_not_found(id))?;
    tx.set_action_payload(
        id,
        &ActionPayload {
            server_targets_json: payload.server_targets_json,
            undo_json: Some(to_json(record)?),
        },
    )
}
