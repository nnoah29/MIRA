//! Apparitions locales provisoires : message déplacé ou étiqueté hors ligne, pas encore vu par le serveur.

use mira_vocabulary::{MailboxId, MessageId, MiraError};
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

/// Les UID provisoires sont strictement inférieurs à `-(2^32)` : aucun UID serveur ni UID à revalider n'y arrive.
const LOCAL_UID_CEILING: i64 = -(1 << 32);

impl Tx<'_> {
    /// Ajoute une apparition provisoire (UID lu comme 0, à revalider) ; sans effet si le message est déjà dans la boîte
    pub fn add_local_appearance(
        &self,
        message: &MessageId,
        mailbox: &MailboxId,
        deleted: bool,
    ) -> Result<(), MiraError> {
        self.require_same_account(message, mailbox)?;
        let uid = self.next_local_uid(mailbox)?;
        self.sql()
            .execute(
                "INSERT OR IGNORE INTO appearances (message_id, mailbox_id, uid, deleted, needs_revalidation) VALUES (?1, ?2, ?3, ?4, 1)",
                params![message.as_str(), mailbox.as_str(), uid, deleted],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Supprime l'apparition provisoire d'un message dans une boîte, si c'en est une
    pub(super) fn remove_local_appearance(
        &self,
        message: &MessageId,
        mailbox: &MailboxId,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM appearances WHERE message_id = ?1 AND mailbox_id = ?2 AND uid < ?3",
                params![message.as_str(), mailbox.as_str(), LOCAL_UID_CEILING],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Prochain UID provisoire libre dans la boîte
    fn next_local_uid(&self, mailbox: &MailboxId) -> Result<i64, MiraError> {
        let lowest: i64 = self
            .sql()
            .query_row(
                "SELECT COALESCE(MIN(uid), ?2) FROM appearances WHERE mailbox_id = ?1 AND uid < ?2",
                params![mailbox.as_str(), LOCAL_UID_CEILING],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        Ok(lowest - 1)
    }
}
