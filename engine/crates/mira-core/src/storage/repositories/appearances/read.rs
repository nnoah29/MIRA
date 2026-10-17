//! Lectures sur les apparitions.

use mira_vocabulary::{Appearance, MailboxId, MessageId, MiraError};
use rusqlite::{OptionalExtension, params};

use super::rows::{COLUMNS, appearance_from_row};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Toutes les apparitions d'un message, par boîte
    pub fn appearances_of_message(
        &self,
        message: &MessageId,
    ) -> Result<Vec<Appearance>, MiraError> {
        let sql =
            format!("SELECT {COLUMNS} FROM appearances WHERE message_id = ?1 ORDER BY mailbox_id");
        let mut statement = self.sql().prepare(&sql).map_err(storage_error)?;
        statement
            .query_map([message.as_str()], appearance_from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)
    }

    /// L'apparition d'un message dans une boîte, si elle existe
    pub fn get_appearance(
        &self,
        message: &MessageId,
        mailbox: &MailboxId,
    ) -> Result<Option<Appearance>, MiraError> {
        let sql =
            format!("SELECT {COLUMNS} FROM appearances WHERE message_id = ?1 AND mailbox_id = ?2");
        self.sql()
            .query_row(
                &sql,
                params![message.as_str(), mailbox.as_str()],
                appearance_from_row,
            )
            .optional()
            .map_err(storage_error)
    }

    /// Toutes les apparitions d'une boîte (supprimées ou non), par message
    pub fn appearances_in_mailbox(
        &self,
        mailbox: &MailboxId,
    ) -> Result<Vec<Appearance>, MiraError> {
        let sql =
            format!("SELECT {COLUMNS} FROM appearances WHERE mailbox_id = ?1 ORDER BY message_id");
        let mut statement = self.sql().prepare(&sql).map_err(storage_error)?;
        statement
            .query_map([mailbox.as_str()], appearance_from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)
    }

    /// L'apparition portant un UID donné dans une boîte, si elle existe
    pub fn appearance_by_uid(
        &self,
        mailbox: &MailboxId,
        uid: u32,
    ) -> Result<Option<Appearance>, MiraError> {
        let sql = format!("SELECT {COLUMNS} FROM appearances WHERE mailbox_id = ?1 AND uid = ?2");
        self.sql()
            .query_row(
                &sql,
                params![mailbox.as_str(), i64::from(uid)],
                appearance_from_row,
            )
            .optional()
            .map_err(storage_error)
    }

    /// Les UID valides d'une boîte avec leur message, par UID croissant
    pub fn uids_in_mailbox(&self, mailbox: &MailboxId) -> Result<Vec<(u32, MessageId)>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT uid, message_id FROM appearances WHERE mailbox_id = ?1 AND needs_revalidation = 0 ORDER BY uid")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([mailbox.as_str()], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?;
        rows.map(|row| {
            let (uid, message) = row.map_err(storage_error)?;
            Ok((
                u32::try_from(uid).unwrap_or(0),
                MessageId::from_raw(message),
            ))
        })
        .collect()
    }
}
