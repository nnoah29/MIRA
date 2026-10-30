//! Complétion de la ligne d'une génération : issue, jetons, outils, types et caractères envoyés en cours de route.

use mira_vocabulary::{AiJournalFinish, ContentKind, MiraError, OperationId};
use rusqlite::{OptionalExtension, params};

use super::errors::ensure_changed;
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::json_columns::{from_json, to_json};

impl Tx<'_> {
    /// Complète la ligne de l'opération ; inconnue : `not_found.operation`
    pub fn finish_ai_journal(
        &self,
        operation: &OperationId,
        finish: &AiJournalFinish,
    ) -> Result<(), MiraError> {
        let Some(kinds) = self.merged_journal_kinds(operation, &finish.extra_kinds)? else {
            return ensure_changed(0);
        };
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_journal SET masked = ?2, input_tokens = ?3, output_tokens = ?4, outcome = ?5, \
                 tools_json = ?6, kinds = ?7, chars = MIN(chars + ?8, 4294967295) WHERE operation = ?1",
                params![
                    operation.as_str(),
                    finish.masked,
                    finish.input_tokens,
                    finish.output_tokens,
                    finish.outcome,
                    to_json(&finish.tools)?,
                    to_json(&kinds)?,
                    finish.extra_chars
                ],
            )
            .map_err(storage_error)?;
        ensure_changed(changed)
    }

    /// Types de la ligne suivis de ceux qui s'y ajoutent, sans doublon ; rien si la ligne n'existe pas
    fn merged_journal_kinds(
        &self,
        operation: &OperationId,
        extra: &[ContentKind],
    ) -> Result<Option<Vec<ContentKind>>, MiraError> {
        let stored: Option<String> = self
            .sql()
            .query_row(
                "SELECT kinds FROM ai_journal WHERE operation = ?1",
                params![operation.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let Some(text) = stored else {
            return Ok(None);
        };
        let mut kinds: Vec<ContentKind> = from_json(&text)?;
        for kind in extra {
            if !kinds.contains(kind) {
                kinds.push(*kind);
            }
        }
        Ok(Some(kinds))
    }
}
