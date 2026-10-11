//! Lectures sur les fils : un fil, ses messages, ses participants.

use mira_vocabulary::{AddressEntry, MessageSummary, MiraError, Thread, ThreadId};
use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, RawThread};
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::json_columns::from_json;
use crate::storage::repositories::messages::MESSAGE_COLUMNS;

impl Tx<'_> {
    /// Relit un fil avec ses compteurs sur tous ses messages, ou rien s'il est inconnu
    pub fn get_thread(&self, id: &ThreadId) -> Result<Option<Thread>, MiraError> {
        let sql = format!(
            "SELECT {COLUMNS} FROM threads t LEFT JOIN messages m ON m.thread_id = t.id WHERE t.id = ?1 GROUP BY t.id"
        );
        let raw = self
            .sql()
            .query_row(&sql, [id.as_str()], RawThread::from_row)
            .optional()
            .map_err(storage_error)?;
        raw.map(|raw| Ok(raw.into_thread(self.participants_of(id)?)))
            .transpose()
    }

    /// Messages d'un fil, du plus ancien au plus récent
    pub fn messages_of_thread(&self, thread: &ThreadId) -> Result<Vec<MessageSummary>, MiraError> {
        let sql = format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.thread_id = ?1 ORDER BY m.sort_date, m.id"
        );
        let rows = self.query_summaries(&sql, [thread.as_str()])?;
        Ok(rows.into_iter().map(|(_, _, summary)| summary).collect())
    }

    /// Expéditeurs distincts d'un fil, dans l'ordre d'apparition
    pub(super) fn participants_of(
        &self,
        thread: &ThreadId,
    ) -> Result<Vec<AddressEntry>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT from_json FROM messages WHERE thread_id = ?1 ORDER BY sort_date, id")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([thread.as_str()], |row| row.get::<_, String>(0))
            .map_err(storage_error)?;
        let mut participants: Vec<AddressEntry> = Vec::new();
        for row in rows {
            for entry in from_json::<Vec<AddressEntry>>(&row.map_err(storage_error)?)? {
                if !participants
                    .iter()
                    .any(|known| known.address == entry.address)
                {
                    participants.push(entry);
                }
            }
        }
        Ok(participants)
    }
}
