//! Mémoire des brouillons de réponse créés par les règles, pour qu'un rejeu ne les double pas.

use mira_vocabulary::{DraftId, MessageId, MiraError};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Brouillon déjà créé pour ce message et ce rang d'action, s'il y en a un
    pub fn rule_reply_draft(
        &self,
        message: &MessageId,
        rank: usize,
    ) -> Result<Option<DraftId>, MiraError> {
        self.sql()
            .query_row(
                "SELECT draft_id FROM rule_reply_drafts WHERE message_id = ?1 AND rank = ?2",
                params![message.as_str(), i64::try_from(rank).unwrap_or(i64::MAX)],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map(|found| found.map(DraftId::from_raw))
            .map_err(storage_error)
    }

    /// Note le brouillon créé pour ce message et ce rang d'action
    pub fn record_rule_reply_draft(
        &self,
        message: &MessageId,
        rank: usize,
        draft: &DraftId,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT OR IGNORE INTO rule_reply_drafts (message_id, rank, draft_id) VALUES (?1, ?2, ?3)",
                params![message.as_str(), i64::try_from(rank).unwrap_or(i64::MAX), draft.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
