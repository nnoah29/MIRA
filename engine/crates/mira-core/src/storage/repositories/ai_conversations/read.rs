//! Lecture des conversations d'IA, de la plus récente à la plus ancienne.

use mira_vocabulary::{AiConversation, AiConversationMeta, MiraError};
use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, meta_from_row, unknown_conversation};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Rend au plus `limit` métadonnées, les plus récentes d'abord ; `before` (identifiant) reprend après cette ligne
    pub fn list_ai_conversations(
        &self,
        limit: u32,
        before: Option<&str>,
    ) -> Result<Vec<AiConversationMeta>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM ai_conversations \
                 WHERE ?1 IS NULL OR (updated_at, rowid) < \
                 (SELECT updated_at, rowid FROM ai_conversations WHERE id = ?1) \
                 ORDER BY updated_at DESC, rowid DESC LIMIT ?2"
            ))
            .map_err(storage_error)?;
        let rows = statement
            .query_map(rusqlite::params![before, limit], meta_from_row)
            .map_err(storage_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)
    }

    /// Rend la conversation complète ; inconnue : `not_found.ai_conversation`
    pub fn get_ai_conversation(&self, id: &str) -> Result<AiConversation, MiraError> {
        let found = self
            .sql()
            .query_row(
                &format!("SELECT {COLUMNS}, payload FROM ai_conversations WHERE id = ?1"),
                [id],
                |row| {
                    Ok(AiConversation {
                        meta: meta_from_row(row)?,
                        payload: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(storage_error)?;
        found.ok_or_else(unknown_conversation)
    }

    /// Rend les métadonnées d'une conversation ; inconnue : `not_found.ai_conversation`
    pub(super) fn require_ai_conversation_meta(
        &self,
        id: &str,
    ) -> Result<AiConversationMeta, MiraError> {
        let found = self
            .sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM ai_conversations WHERE id = ?1"),
                [id],
                meta_from_row,
            )
            .optional()
            .map_err(storage_error)?;
        found.ok_or_else(unknown_conversation)
    }
}
