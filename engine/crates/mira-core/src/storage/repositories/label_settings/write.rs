//! Écritures sur les réglages des libellés.

use mira_vocabulary::{LabelRef, LabelSettings, MiraError};
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Enregistre (crée ou remplace) les réglages d'un libellé
    pub fn put_label_settings(
        &self,
        label: &LabelRef,
        settings: &LabelSettings,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT INTO label_settings (kind, label_id, hidden, hide_from_inbox, notifications, counter, color, position) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
                 ON CONFLICT (kind, label_id) DO UPDATE SET hidden = excluded.hidden, hide_from_inbox = excluded.hide_from_inbox, \
                 notifications = excluded.notifications, counter = excluded.counter, color = excluded.color, position = excluded.position",
                params![
                    label.kind.as_str(),
                    label.id,
                    settings.hidden,
                    settings.hide_from_inbox,
                    settings.notifications,
                    settings.counter.as_str(),
                    settings.color,
                    settings.position,
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Supprime les réglages d'un libellé (sans effet s'il n'en a pas)
    pub fn delete_label_settings(&self, label: &LabelRef) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM label_settings WHERE kind = ?1 AND label_id = ?2",
                params![label.kind.as_str(), label.id],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
