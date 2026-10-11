//! Charge utile d'une action : cibles serveur mémorisées et inverse de l'effet local (JSON opaque pour le dépôt).

use mira_vocabulary::{ActionId, MiraError, Timestamp};
use rusqlite::{OptionalExtension, params};

use super::rows::{action_not_found, state_name};
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use mira_vocabulary::ActionState;

/// Charge utile brute d'une action, telle que stockée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPayload {
    pub server_targets_json: String,
    pub undo_json: Option<String>,
}

impl Tx<'_> {
    /// Enregistre les cibles serveur et l'inverse de l'effet local d'une action
    pub fn set_action_payload(
        &self,
        id: &ActionId,
        payload: &ActionPayload,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE actions SET server_targets_json = ?2, undo_json = ?3 WHERE id = ?1",
                params![id.as_str(), payload.server_targets_json, payload.undo_json],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(action_not_found(id));
        }
        Ok(())
    }

    /// Relit la charge utile d'une action, ou rien si l'action est inconnue
    pub fn get_action_payload(&self, id: &ActionId) -> Result<Option<ActionPayload>, MiraError> {
        self.sql()
            .query_row(
                "SELECT server_targets_json, undo_json FROM actions WHERE id = ?1",
                [id.as_str()],
                |row| {
                    Ok(ActionPayload {
                        server_targets_json: row.get(0)?,
                        undo_json: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(storage_error)
    }

    /// Remet une action échouée en attente : tentatives à zéro, erreur effacée, prochaine date d'essai
    pub fn requeue_action(&self, id: &ActionId, not_before: Timestamp) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE actions SET state = ?2, attempts = 0, last_error = NULL, not_before = ?3 WHERE id = ?1",
                params![id.as_str(), state_name(ActionState::Pending), not_before.as_millis()],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(action_not_found(id));
        }
        Ok(())
    }
}
