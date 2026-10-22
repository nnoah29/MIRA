//! Écritures sur les libellés locaux : création, renommage, couleur, suppression.

use mira_vocabulary::{LocalLabel, LocalLabelId, MiraError};
use rusqlite::{OptionalExtension, params};

use super::rows::{label_not_found, name_key, name_taken};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Insère un libellé ; un nom déjà pris (casse ignorée) est refusé
    pub fn insert_local_label(&self, label: &LocalLabel) -> Result<(), MiraError> {
        self.ensure_name_free(&label.name, None)?;
        self.sql()
            .execute(
                "INSERT INTO local_labels (id, name, name_key, color) VALUES (?1, ?2, ?3, ?4)",
                params![
                    label.id.as_str(),
                    label.name.trim(),
                    name_key(&label.name),
                    label.color
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Renomme un libellé ; inconnu : `not_found.local_label` ; nom pris par un autre : refusé
    pub fn rename_local_label(&self, id: &LocalLabelId, name: &str) -> Result<(), MiraError> {
        self.ensure_name_free(name, Some(id))?;
        let changed = self
            .sql()
            .execute(
                "UPDATE local_labels SET name = ?2, name_key = ?3 WHERE id = ?1",
                params![id.as_str(), name.trim(), name_key(name)],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(label_not_found());
        }
        Ok(())
    }

    /// Change la couleur d'un libellé
    pub fn recolor_local_label(&self, id: &LocalLabelId, color: u32) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE local_labels SET color = ?2 WHERE id = ?1",
                params![id.as_str(), color],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(label_not_found());
        }
        Ok(())
    }

    /// Supprime un libellé et toutes ses poses
    pub fn delete_local_label(&self, id: &LocalLabelId) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute("DELETE FROM local_labels WHERE id = ?1", [id.as_str()])
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(label_not_found());
        }
        self.sql()
            .execute(
                "DELETE FROM label_settings WHERE kind = 'local' AND label_id = ?1",
                [id.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Refuse un nom déjà porté par un autre libellé que `except`
    fn ensure_name_free(&self, name: &str, except: Option<&LocalLabelId>) -> Result<(), MiraError> {
        let owner: Option<String> = self
            .sql()
            .query_row(
                "SELECT id FROM local_labels WHERE name_key = ?1",
                [name_key(name)],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        match owner {
            Some(found) if except.is_none_or(|id| id.as_str() != found) => Err(name_taken()),
            _ => Ok(()),
        }
    }
}
