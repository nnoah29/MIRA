//! Écritures sur les boîtes : upsert, repère de synchronisation, suppression, compteurs.

use mira_vocabulary::{Mailbox, MailboxId, MiraError, SyncMarker};
use rusqlite::{params, params_from_iter};

use super::rows::{mailbox_not_found, mailbox_values};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

const UPSERT: &str = "INSERT INTO mailboxes (id, account_id, name, display_name, delimiter, parent_id, role, sync_enabled, uid_validity, uid_next, highest_modseq, total, unread) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13) ON CONFLICT (account_id, name) DO UPDATE SET display_name = excluded.display_name, delimiter = excluded.delimiter, parent_id = excluded.parent_id, role = excluded.role, sync_enabled = excluded.sync_enabled";

/// Requête des compteurs : la Réception ignore les messages masqués par un libellé
fn counters_sql() -> String {
    let visible = format!(
        "(mailboxes.role <> 'inbox' OR NOT {})",
        crate::storage::repositories::label_settings::hidden::hidden_from_inbox()
    );
    format!(
        "UPDATE mailboxes SET \
         total = (SELECT COUNT(*) FROM appearances a JOIN messages m ON m.id = a.message_id WHERE a.mailbox_id = mailboxes.id AND a.deleted = 0 AND {visible}), \
         unread = (SELECT COUNT(*) FROM appearances a JOIN messages m ON m.id = a.message_id WHERE a.mailbox_id = mailboxes.id AND a.deleted = 0 AND m.seen = 0 AND {visible}) \
         WHERE id = ?1"
    )
}

const MARKER: &str =
    "UPDATE mailboxes SET uid_validity = ?2, uid_next = ?3, highest_modseq = ?4 WHERE id = ?1";

impl Tx<'_> {
    /// Insère ou met à jour une boîte par (compte, nom) et rend l'identifiant retenu
    pub fn upsert_mailbox(&self, mailbox: &Mailbox) -> Result<MailboxId, MiraError> {
        self.sql()
            .execute(UPSERT, params_from_iter(mailbox_values(mailbox)))
            .map_err(storage_error)?;
        self.sql()
            .query_row(
                "SELECT id FROM mailboxes WHERE account_id = ?1 AND name = ?2",
                params![mailbox.account.as_str(), mailbox.name],
                |row| row.get::<_, String>(0),
            )
            .map(MailboxId::from_raw)
            .map_err(storage_error)
    }

    /// Enregistre (ou efface) le repère de synchronisation d'une boîte
    pub fn set_mailbox_marker(
        &self,
        id: &MailboxId,
        marker: Option<SyncMarker>,
    ) -> Result<(), MiraError> {
        let modseq = marker.and_then(|m| m.highest_modseq.and_then(|v| i64::try_from(v).ok()));
        let validity = marker.map(|m| i64::from(m.uid_validity));
        let next = marker.map(|m| i64::from(m.uid_next));
        let changed = self
            .sql()
            .execute(MARKER, params![id.as_str(), validity, next, modseq])
            .map_err(storage_error)?;
        require_change(changed, id)
    }

    /// Supprime une boîte et, en cascade, ses apparitions
    pub fn delete_mailbox(&self, id: &MailboxId) -> Result<(), MiraError> {
        self.sql()
            .execute("DELETE FROM mailboxes WHERE id = ?1", [id.as_str()])
            .map_err(storage_error)?;
        self.sql()
            .execute(
                "DELETE FROM label_settings WHERE kind = 'server' AND label_id = ?1",
                [id.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Recalcule le total et les non lus depuis les apparitions non supprimées
    pub fn refresh_mailbox_counters(&self, id: &MailboxId) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(&counters_sql(), [id.as_str()])
            .map_err(storage_error)?;
        require_change(changed, id)
    }
}

/// Échoue avec « boîte inconnue » si l'écriture n'a touché aucune ligne
fn require_change(changed: usize, id: &MailboxId) -> Result<(), MiraError> {
    if changed == 0 {
        return Err(mailbox_not_found(id));
    }
    Ok(())
}
