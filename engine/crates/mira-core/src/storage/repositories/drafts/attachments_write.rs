//! Écritures sur les pièces jointes d'un brouillon.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{DraftId, ErrorCategory, MiraError};
use rusqlite::params;

use super::rows::draft_not_found;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Ajoute une pièce jointe à un brouillon et rend sa position, stable jusqu'au retrait
    pub fn add_draft_attachment(
        &self,
        draft: &DraftId,
        filename: &str,
        mime_type: &str,
        data: &[u8],
    ) -> Result<u32, MiraError> {
        if self.get_draft(draft)?.is_none() {
            return Err(draft_not_found(draft));
        }
        let position: u32 = self
            .sql()
            .query_row(
                "SELECT COALESCE(MAX(position) + 1, 0) FROM draft_attachments WHERE draft_id = ?1",
                [draft.as_str()],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        self.sql()
            .execute(
                "INSERT INTO draft_attachments (draft_id, position, filename, mime_type, data) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![draft.as_str(), position, filename, mime_type, data],
            )
            .map_err(storage_error)?;
        Ok(position)
    }

    /// Retire une pièce jointe d'un brouillon par sa position
    pub fn remove_draft_attachment(&self, draft: &DraftId, position: u32) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "DELETE FROM draft_attachments WHERE draft_id = ?1 AND position = ?2",
                params![draft.as_str(), position],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(MiraError::new(
                ErrorCategory::NotFound,
                not_found::ATTACHMENT,
                "pièce jointe de brouillon inconnue",
            ));
        }
        Ok(())
    }
}
