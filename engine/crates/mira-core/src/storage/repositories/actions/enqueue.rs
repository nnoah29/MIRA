//! Mise en file idempotente d'une action (invariants 17 et 18).

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError, PendingAction};
use rusqlite::params;

use super::rows::state_name;
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::json_columns::to_json;

impl Tx<'_> {
    /// Met l'action en file ; la même clé pour le même compte rend l'action déjà enregistrée
    pub fn enqueue_action(&self, action: &PendingAction) -> Result<PendingAction, MiraError> {
        let existing =
            self.find_action_by_idempotency_key(&action.account, &action.idempotency_key)?;
        if let Some(existing) = existing {
            ensure_same_command(&existing, action)?;
            return Ok(existing);
        }
        self.sql()
            .execute(
                "INSERT INTO actions (id, account_id, kind_json, targets_json, server_targets_json, idempotency_key, state, attempts, last_error, created_at, not_before) VALUES (?1, ?2, ?3, ?4, '[]', ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    action.id.as_str(),
                    action.account.as_str(),
                    to_json(&action.kind)?,
                    to_json(&action.targets)?,
                    action.idempotency_key.0,
                    state_name(action.state),
                    action.attempts,
                    action.last_error,
                    action.created_at.as_millis(),
                    action.not_before.as_millis(),
                ],
            )
            .map_err(storage_error)?;
        Ok(action.clone())
    }
}

/// Refuse la réutilisation d'une clé d'idempotence pour une autre commande
fn ensure_same_command(existing: &PendingAction, wanted: &PendingAction) -> Result<(), MiraError> {
    if existing.kind == wanted.kind && existing.targets == wanted.targets {
        return Ok(());
    }
    Err(MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::IDEMPOTENCY_CONFLICT,
        "clé d'idempotence déjà utilisée pour une autre commande",
    ))
}
