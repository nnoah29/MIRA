//! États d'une apparition : suppression serveur et revalidation après changement d'UIDVALIDITY (D-12).

use mira_vocabulary::{MailboxId, MessageId, MiraError};
use rusqlite::params;

use super::rows::appearance_not_found;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Marque (ou démarque) l'apparition comme supprimée sur le serveur
    pub fn set_appearance_deleted(
        &self,
        message: &MessageId,
        mailbox: &MailboxId,
        deleted: bool,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE appearances SET deleted = ?3 WHERE message_id = ?1 AND mailbox_id = ?2",
                params![message.as_str(), mailbox.as_str(), deleted],
            )
            .map_err(storage_error)?;
        require_appearance(changed, message, mailbox)
    }

    /// Marque toutes les apparitions de la boîte à revalider ; leur UID devient négatif pour libérer l'espace des UID
    pub fn mark_mailbox_for_revalidation(&self, mailbox: &MailboxId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "UPDATE appearances SET needs_revalidation = 1, uid = -ABS(uid) WHERE mailbox_id = ?1",
                [mailbox.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Confirme une apparition sous son nouvel UID : le message garde son identifiant interne
    pub fn revalidate_appearance(
        &self,
        message: &MessageId,
        mailbox: &MailboxId,
        new_uid: u32,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE appearances SET uid = ?3, needs_revalidation = 0 WHERE message_id = ?1 AND mailbox_id = ?2",
                params![message.as_str(), mailbox.as_str(), i64::from(new_uid)],
            )
            .map_err(storage_error)?;
        require_appearance(changed, message, mailbox)
    }
}

/// Échoue avec « apparition inconnue » si l'écriture n'a touché aucune ligne
fn require_appearance(
    changed: usize,
    message: &MessageId,
    mailbox: &MailboxId,
) -> Result<(), MiraError> {
    if changed == 0 {
        return Err(appearance_not_found(message, mailbox));
    }
    Ok(())
}
