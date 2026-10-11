//! Renommage d'une boîte et des boîtes qu'elle contient dans la hiérarchie.

use mira_vocabulary::{MailboxId, MiraError};
use rusqlite::params;

use super::rows::mailbox_not_found;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Renomme une boîte et remplace le préfixe du nom de ses descendantes ; rend l'ancien nom
    pub fn rename_mailbox(
        &self,
        id: &MailboxId,
        new_name: &str,
        new_display_name: &str,
    ) -> Result<String, MiraError> {
        let mailbox = self.get_mailbox(id)?.ok_or_else(|| mailbox_not_found(id))?;
        let delimiter = mailbox.delimiter.clone().unwrap_or_default();
        if !delimiter.is_empty() {
            self.sql()
                .execute(
                    "UPDATE mailboxes SET name = ?3 || substr(name, length(?2) + 1) WHERE account_id = ?1 AND substr(name, 1, length(?2) + length(?4)) = ?2 || ?4",
                    params![mailbox.account.as_str(), mailbox.name, new_name, delimiter],
                )
                .map_err(storage_error)?;
        }
        self.sql()
            .execute(
                "UPDATE mailboxes SET name = ?2, display_name = ?3 WHERE id = ?1",
                params![id.as_str(), new_name, new_display_name],
            )
            .map_err(storage_error)?;
        Ok(mailbox.name)
    }
}
