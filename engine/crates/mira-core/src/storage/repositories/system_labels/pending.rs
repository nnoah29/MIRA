//! Conversations à réévaluer : arrivée d'un message, échéance d'un délai, ou libellé automatique déjà posé.

use mira_vocabulary::{AccountId, MessageId, MiraError, ThreadId};
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Marque une conversation comme à réévaluer (un message vient d'y être rattaché)
    pub fn mark_thread_labels_dirty(&self, thread: &ThreadId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "UPDATE threads SET labels_dirty = 1 WHERE id = ?1",
                [thread.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Marque comme à réévaluer la conversation d'un message dont le corps vient d'être rangé
    pub fn mark_message_thread_labels_dirty(&self, message: &MessageId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "UPDATE threads SET labels_dirty = 1 WHERE id = (SELECT thread_id FROM messages WHERE id = ?1)",
                [message.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Conversations d'un compte à réévaluer à cet instant, au plus `limit`
    pub fn threads_needing_labels(
        &self,
        account: &AccountId,
        now: i64,
        limit: u32,
    ) -> Result<Vec<ThreadId>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(
                "SELECT t.id FROM threads t WHERE t.account_id = ?1 AND (t.labels_dirty = 1 \
                 OR (t.labels_recheck_at IS NOT NULL AND t.labels_recheck_at <= ?2) \
                 OR EXISTS (SELECT 1 FROM thread_system_labels s WHERE s.account_id = t.account_id \
                            AND s.root_key = t.root_key AND s.on_state = 1 AND s.source = 'auto')) \
                 ORDER BY t.labels_dirty DESC, t.id LIMIT ?3",
            )
            .map_err(storage_error)?;
        statement
            .query_map(params![account.as_str(), now, limit], |row| {
                Ok(ThreadId::from_raw(row.get::<_, String>(0)?))
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)
    }

    /// Note qu'une conversation est à jour, avec l'instant où son verdict peut changer sans nouveau message
    pub fn settle_thread_labels(
        &self,
        thread: &ThreadId,
        recheck_at: Option<i64>,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "UPDATE threads SET labels_dirty = 0, labels_recheck_at = ?2 WHERE id = ?1",
                params![thread.as_str(), recheck_at],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
