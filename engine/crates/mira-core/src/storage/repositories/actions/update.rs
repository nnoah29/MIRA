//! Changements d'état et échecs d'une action en file.

use mira_vocabulary::{ActionId, ActionState, MiraError, Timestamp};
use rusqlite::params;

use super::rows::{action_not_found, state_name};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Change l'état d'une action
    pub fn set_action_state(&self, id: &ActionId, state: ActionState) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE actions SET state = ?2 WHERE id = ?1",
                params![id.as_str(), state_name(state)],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(action_not_found(id));
        }
        Ok(())
    }

    /// Note un échec : une tentative de plus, dernière erreur et prochaine date d'essai
    pub fn record_action_failure(
        &self,
        id: &ActionId,
        error: &str,
        not_before: Timestamp,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE actions SET attempts = attempts + 1, last_error = ?2, not_before = ?3 WHERE id = ?1",
                params![id.as_str(), error, not_before.as_millis()],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(action_not_found(id));
        }
        Ok(())
    }
}
