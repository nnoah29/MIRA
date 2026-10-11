//! Lectures sur les pièces jointes d'un brouillon.

use mira_vocabulary::{DraftAttachment, DraftId, MiraError};
use rusqlite::OptionalExtension;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Liste les métadonnées des pièces jointes d'un brouillon, dans l'ordre d'ajout
    pub fn list_draft_attachments(
        &self,
        draft: &DraftId,
    ) -> Result<Vec<DraftAttachment>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT filename, mime_type, length(data) FROM draft_attachments WHERE draft_id = ?1 ORDER BY position")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([draft.as_str()], |row| {
                Ok(DraftAttachment {
                    filename: row.get(0)?,
                    mime_type: row.get(1)?,
                    size: u64::try_from(row.get::<_, i64>(2)?).unwrap_or(0),
                })
            })
            .map_err(storage_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)
    }

    /// Relit le contenu d'une pièce jointe de brouillon, ou rien si la position est inconnue
    pub fn read_draft_attachment_data(
        &self,
        draft: &DraftId,
        position: u32,
    ) -> Result<Option<Vec<u8>>, MiraError> {
        self.sql()
            .query_row(
                "SELECT data FROM draft_attachments WHERE draft_id = ?1 AND position = ?2",
                rusqlite::params![draft.as_str(), position],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)
    }
}
