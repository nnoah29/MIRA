//! Lectures sur les pièces jointes.

use mira_vocabulary::{Attachment, AttachmentId, MessageId, MiraError};
use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, attachment_from_row};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Liste les pièces jointes d'un message, dans l'ordre des parties
    pub fn list_attachments(&self, message: &MessageId) -> Result<Vec<Attachment>, MiraError> {
        let sql =
            format!("SELECT {COLUMNS} FROM attachments WHERE message_id = ?1 ORDER BY part_index");
        let mut statement = self.sql().prepare(&sql).map_err(storage_error)?;
        statement
            .query_map([message.as_str()], attachment_from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)
    }

    /// Relit une pièce jointe, ou rien si elle est inconnue
    pub fn get_attachment(&self, id: &AttachmentId) -> Result<Option<Attachment>, MiraError> {
        let sql = format!("SELECT {COLUMNS} FROM attachments WHERE id = ?1");
        self.sql()
            .query_row(&sql, [id.as_str()], attachment_from_row)
            .optional()
            .map_err(storage_error)
    }

    /// Position de la partie MIME d'une pièce jointe dans son message
    pub fn attachment_part_index(&self, id: &AttachmentId) -> Result<Option<u32>, MiraError> {
        self.sql()
            .query_row(
                "SELECT part_index FROM attachments WHERE id = ?1",
                [id.as_str()],
                |row| row.get::<_, u32>(0),
            )
            .optional()
            .map_err(storage_error)
    }

    /// Chemin du contenu téléchargé d'une pièce jointe, s'il existe
    pub fn attachment_cache_path(&self, id: &AttachmentId) -> Result<Option<String>, MiraError> {
        let path = self
            .sql()
            .query_row(
                "SELECT cache_path FROM attachments WHERE id = ?1",
                [id.as_str()],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(path.flatten())
    }
}
