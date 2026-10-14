//! Écritures sur les pièces jointes.

use mira_vocabulary::{Attachment, AttachmentId, MiraError};
use rusqlite::params;

use super::rows::attachment_not_found;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Insère une pièce jointe à sa position dans le message ; l'état du cache vient de `set_attachment_cache_path`
    pub fn insert_attachment(
        &self,
        attachment: &Attachment,
        part_index: u32,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT INTO attachments (id, message_id, part_index, filename, mime_type, size, content_id, inline) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    attachment.id.as_str(),
                    attachment.message.as_str(),
                    part_index,
                    attachment.filename,
                    attachment.mime_type,
                    i64::try_from(attachment.size).unwrap_or(i64::MAX),
                    attachment.content_id,
                    attachment.inline
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Oublie tous les chemins de cache et les rend, pour que l'appelant supprime les fichiers
    pub fn clear_attachment_cache_paths(&self) -> Result<Vec<String>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT cache_path FROM attachments WHERE cache_path IS NOT NULL")
            .map_err(storage_error)?;
        let paths = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        self.sql()
            .execute("UPDATE attachments SET cache_path = NULL", [])
            .map_err(storage_error)?;
        Ok(paths)
    }

    /// Enregistre (ou efface) le chemin du contenu téléchargé d'une pièce jointe
    pub fn set_attachment_cache_path(
        &self,
        id: &AttachmentId,
        path: Option<&str>,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE attachments SET cache_path = ?2 WHERE id = ?1",
                params![id.as_str(), path],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(attachment_not_found());
        }
        Ok(())
    }
}
