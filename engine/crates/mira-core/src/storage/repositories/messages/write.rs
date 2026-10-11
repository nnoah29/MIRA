//! Insertion et suppression de messages.

use mira_vocabulary::{MessageId, MessageSummary, MiraError, Timestamp};
use rusqlite::params_from_iter;

use super::values::{INSERT_SQL, header_values, state_values};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Insère un message ; ses apparitions s'ajoutent séparément
    pub fn insert_message(
        &self,
        summary: &MessageSummary,
        sort_date: Timestamp,
    ) -> Result<(), MiraError> {
        let mut values = header_values(summary)?;
        values.extend(state_values(summary, sort_date));
        self.sql()
            .execute(INSERT_SQL, params_from_iter(values))
            .map_err(storage_error)?;
        Ok(())
    }

    /// Supprime un message du miroir, avec son index de recherche et tout ce qui en dépend
    pub fn delete_message(&self, id: &MessageId) -> Result<(), MiraError> {
        self.unindex_message(id)?;
        self.sql()
            .execute("DELETE FROM messages WHERE id = ?1", [id.as_str()])
            .map_err(storage_error)?;
        Ok(())
    }
}
