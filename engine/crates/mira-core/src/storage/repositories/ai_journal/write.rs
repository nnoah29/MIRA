//! Écritures sur le journal des envois d'IA : ouverture, complétion, vidage, rétention, reprise.

use mira_vocabulary::{AiJournalEntry, AiJournalFinish, MiraError, OperationId, Timestamp};
use rusqlite::params;

use super::errors::ensure_changed;
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::json_columns::to_json;

impl Tx<'_> {
    /// Ouvre la ligne d'une génération ; l'opération est unique dans le journal
    pub fn insert_ai_journal(&self, entry: &AiJournalEntry) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT INTO ai_journal (id, at, operation, connection, provider, model, local, kinds, \
                 parts, chars, masked, input_tokens, output_tokens, outcome) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    entry.id,
                    entry.at.as_millis(),
                    entry.operation.as_str(),
                    entry.connection.as_str(),
                    entry.provider.0,
                    entry.model,
                    entry.local,
                    to_json(&entry.kinds)?,
                    entry.parts,
                    entry.chars,
                    entry.masked,
                    entry.input_tokens,
                    entry.output_tokens,
                    entry.outcome
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Complète la ligne de l'opération ; inconnue : `not_found.operation`
    pub fn finish_ai_journal(
        &self,
        operation: &OperationId,
        finish: &AiJournalFinish,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_journal SET masked = ?2, input_tokens = ?3, output_tokens = ?4, outcome = ?5 \
                 WHERE operation = ?1",
                params![
                    operation.as_str(),
                    finish.masked,
                    finish.input_tokens,
                    finish.output_tokens,
                    finish.outcome
                ],
            )
            .map_err(storage_error)?;
        ensure_changed(changed)
    }

    /// Vide le journal
    pub fn clear_ai_journal(&self) -> Result<(), MiraError> {
        self.sql()
            .execute("DELETE FROM ai_journal", [])
            .map_err(storage_error)?;
        Ok(())
    }

    /// Garde les `keep` lignes les plus récentes et rien d'antérieur à `not_before` ; une ligne `running` reste
    pub fn trim_ai_journal(&self, keep: usize, not_before: Timestamp) -> Result<u64, MiraError> {
        let keep = i64::try_from(keep).unwrap_or(i64::MAX);
        let removed = self
            .sql()
            .execute(
                "DELETE FROM ai_journal WHERE outcome <> 'running' AND (at < ?2 OR rowid NOT IN \
                 (SELECT rowid FROM ai_journal ORDER BY at DESC, rowid DESC LIMIT ?1))",
                params![keep, not_before.as_millis()],
            )
            .map_err(storage_error)?;
        Ok(u64::try_from(removed).unwrap_or(0))
    }

    /// Marque `interrupted` les lignes restées `running` d'un arrêt brutal et rend leur nombre
    pub fn interrupt_running_ai_journal(&self) -> Result<usize, MiraError> {
        self.sql()
            .execute(
                "UPDATE ai_journal SET outcome = 'interrupted' WHERE outcome = 'running'",
                [],
            )
            .map_err(storage_error)
    }
}
