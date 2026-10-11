//! Purge des apparitions non revalidées et des messages orphelins (invariant 8).

use mira_vocabulary::{AccountId, MailboxId, MessageId, MiraError};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

const ORPHANED_BY_PURGE: &str = "SELECT a.message_id FROM appearances a WHERE a.mailbox_id = ?1 AND a.needs_revalidation = 1 AND NOT EXISTS (SELECT 1 FROM appearances b WHERE b.message_id = a.message_id AND b.mailbox_id <> ?1)";

const ORPHANS_OF_ACCOUNT: &str = "SELECT m.id FROM messages m WHERE m.account_id = ?1 AND NOT EXISTS (SELECT 1 FROM appearances a WHERE a.message_id = m.id)";

impl Tx<'_> {
    /// Supprime les apparitions non revalidées et rend les messages qui n'en ont plus aucune
    pub fn purge_unrevalidated(&self, mailbox: &MailboxId) -> Result<Vec<MessageId>, MiraError> {
        let orphans = self.message_ids(ORPHANED_BY_PURGE, mailbox.as_str())?;
        self.sql()
            .execute(
                "DELETE FROM appearances WHERE mailbox_id = ?1 AND needs_revalidation = 1",
                [mailbox.as_str()],
            )
            .map_err(storage_error)?;
        Ok(orphans)
    }

    /// Supprime du miroir les messages d'un compte sans aucune apparition et rend leurs identifiants
    pub fn delete_orphan_messages(&self, account: &AccountId) -> Result<Vec<MessageId>, MiraError> {
        let orphans = self.message_ids(ORPHANS_OF_ACCOUNT, account.as_str())?;
        for id in &orphans {
            self.delete_message(id)?;
        }
        Ok(orphans)
    }

    /// Lit une colonne d'identifiants de messages pour un paramètre texte
    fn message_ids(&self, sql: &str, parameter: &str) -> Result<Vec<MessageId>, MiraError> {
        let mut statement = self.sql().prepare(sql).map_err(storage_error)?;
        let rows = statement
            .query_map([parameter], |row| row.get::<_, String>(0))
            .map_err(storage_error)?;
        rows.map(|row| row.map(MessageId::from_raw).map_err(storage_error))
            .collect()
    }
}
