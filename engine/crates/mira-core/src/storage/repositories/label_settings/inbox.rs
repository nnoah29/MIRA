//! Effets des réglages de libellés sur la Réception : compteurs recalculés, message silencieux.

use mira_vocabulary::{Mailbox, MailboxRole, MessageId, MiraError};

use super::hidden::{hidden_from_inbox, notifications_muted};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Vrai si au moins un libellé est masqué de la Réception
    pub fn any_hidden_from_inbox(&self) -> Result<bool, MiraError> {
        self.sql()
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM label_settings WHERE hide_from_inbox = 1)",
                [],
                |row| row.get(0),
            )
            .map_err(storage_error)
    }

    /// Recalcule les compteurs de chaque Réception ; sans `force`, seulement si un libellé est masqué
    pub fn refresh_inbox_counters(&self, force: bool) -> Result<Vec<Mailbox>, MiraError> {
        if !force && !self.any_hidden_from_inbox()? {
            return Ok(Vec::new());
        }
        let mut refreshed = Vec::new();
        for account in self.list_accounts()? {
            for mailbox in self.list_mailboxes(&account.id)? {
                if mailbox.role == MailboxRole::Inbox {
                    self.refresh_mailbox_counters(&mailbox.id)?;
                    refreshed.extend(self.get_mailbox(&mailbox.id)?);
                }
            }
        }
        Ok(refreshed)
    }

    /// Vrai si le message est masqué de la Réception ou porte un libellé aux notifications coupées
    pub fn label_silences(&self, message: &MessageId) -> Result<bool, MiraError> {
        let sql = format!(
            "SELECT EXISTS (SELECT 1 FROM messages m WHERE m.id = ?1 AND ({} OR {}))",
            hidden_from_inbox(),
            notifications_muted()
        );
        self.sql()
            .query_row(&sql, [message.as_str()], |row| row.get(0))
            .map_err(storage_error)
    }
}
