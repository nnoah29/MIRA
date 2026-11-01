//! Écritures sur les conversations d'IA : enregistrement, renommage, suppression, effacement, rétention.

use mira_vocabulary::{AiConversationMeta, MiraError};
use rusqlite::params;

use super::rows::unknown_conversation;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Crée ou remplace le contenu ; date de création et titre d'une conversation existante sont conservés
    pub fn save_ai_conversation(
        &self,
        id: &str,
        title: &str,
        payload_version: u32,
        payload: &str,
        now: i64,
    ) -> Result<AiConversationMeta, MiraError> {
        self.sql()
            .execute(
                "INSERT INTO ai_conversations (id, title, created_at, updated_at, payload_version, payload) \
                 VALUES (?1, ?2, ?5, ?5, ?3, ?4) \
                 ON CONFLICT (id) DO UPDATE SET updated_at = excluded.updated_at, \
                 payload_version = excluded.payload_version, payload = excluded.payload",
                params![id, title, payload_version, payload, now],
            )
            .map_err(storage_error)?;
        self.require_ai_conversation_meta(id)
    }

    /// Change le titre sans toucher la date de mise à jour ; inconnue : `not_found.ai_conversation`
    pub fn rename_ai_conversation(
        &self,
        id: &str,
        title: &str,
    ) -> Result<AiConversationMeta, MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_conversations SET title = ?2 WHERE id = ?1",
                params![id, title],
            )
            .map_err(storage_error)?;
        match changed {
            0 => Err(unknown_conversation()),
            _ => self.require_ai_conversation_meta(id),
        }
    }

    /// Supprime la conversation ; inconnue : `not_found.ai_conversation`
    pub fn delete_ai_conversation(&self, id: &str) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute("DELETE FROM ai_conversations WHERE id = ?1", [id])
            .map_err(storage_error)?;
        match changed {
            0 => Err(unknown_conversation()),
            _ => Ok(()),
        }
    }

    /// Supprime toutes les conversations
    pub fn clear_ai_conversations(&self) -> Result<(), MiraError> {
        self.sql()
            .execute("DELETE FROM ai_conversations", [])
            .map_err(storage_error)?;
        Ok(())
    }

    /// Garde les `keep` plus récentes et la conversation `protect` ; rend le nombre supprimé
    pub fn trim_ai_conversations(&self, keep: usize, protect: &str) -> Result<u64, MiraError> {
        let keep = i64::try_from(keep).unwrap_or(i64::MAX);
        let removed = self
            .sql()
            .execute(
                "DELETE FROM ai_conversations WHERE id <> ?2 AND rowid NOT IN \
                 (SELECT rowid FROM ai_conversations ORDER BY updated_at DESC, rowid DESC LIMIT ?1)",
                params![keep, protect],
            )
            .map_err(storage_error)?;
        Ok(u64::try_from(removed).unwrap_or(0))
    }
}
