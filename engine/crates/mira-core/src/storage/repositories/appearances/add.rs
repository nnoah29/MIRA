//! Ajout et retrait d'une apparition, avec contrôle du compte (invariant 9).

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{Appearance, ErrorCategory, MailboxId, MessageId, MiraError};
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::repositories::mailboxes::mailbox_not_found;
use crate::storage::repositories::messages::message_not_found;

impl Tx<'_> {
    /// Ajoute une apparition ; la boîte doit appartenir au compte du message
    pub fn add_appearance(&self, appearance: &Appearance) -> Result<(), MiraError> {
        self.require_same_account(&appearance.message, &appearance.mailbox)?;
        self.sql()
            .execute(
                "INSERT INTO appearances (message_id, mailbox_id, uid, deleted, needs_revalidation) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    appearance.message.as_str(),
                    appearance.mailbox.as_str(),
                    i64::from(appearance.uid),
                    appearance.deleted,
                    appearance.needs_revalidation
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Retire l'apparition d'un message dans une boîte (sans effet si elle n'existe pas)
    pub fn remove_appearance(
        &self,
        message: &MessageId,
        mailbox: &MailboxId,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM appearances WHERE message_id = ?1 AND mailbox_id = ?2",
                params![message.as_str(), mailbox.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Refuse une apparition dont la boîte et le message sont de comptes différents
    fn require_same_account(
        &self,
        message: &MessageId,
        mailbox: &MailboxId,
    ) -> Result<(), MiraError> {
        let accounts = self
            .sql()
            .query_row(
                "SELECT (SELECT account_id FROM messages WHERE id = ?1), (SELECT account_id FROM mailboxes WHERE id = ?2)",
                params![message.as_str(), mailbox.as_str()],
                |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .map_err(storage_error)?;
        match accounts {
            (None, _) => Err(message_not_found(message)),
            (_, None) => Err(mailbox_not_found(mailbox)),
            (Some(left), Some(right)) if left == right => Ok(()),
            _ => Err(MiraError::new(
                ErrorCategory::InvalidInput,
                invalid_input::PARAMETER,
                "la boîte et le message sont de comptes différents",
            )),
        }
    }
}
