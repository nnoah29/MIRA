//! Lectures sur les brouillons.

use mira_vocabulary::{AccountId, Draft, DraftId, MiraError};
use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, RawDraft};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit un brouillon avec les métadonnées de ses pièces jointes, ou rien s'il n'existe pas
    pub fn get_draft(&self, id: &DraftId) -> Result<Option<Draft>, MiraError> {
        let raw = self
            .sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM drafts WHERE id = ?1"),
                [id.as_str()],
                RawDraft::from_row,
            )
            .optional()
            .map_err(storage_error)?;
        match raw {
            Some(raw) => Ok(Some(raw.into_draft(self.list_draft_attachments(id)?)?)),
            None => Ok(None),
        }
    }

    /// Liste les brouillons d'un compte, le plus récemment modifié d'abord
    pub fn list_drafts(&self, account: &AccountId) -> Result<Vec<Draft>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM drafts WHERE account_id = ?1 ORDER BY updated_at DESC, id"
            ))
            .map_err(storage_error)?;
        let raws = statement
            .query_map([account.as_str()], RawDraft::from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        raws.into_iter()
            .map(|raw| {
                let id = DraftId::from_raw(raw.id.clone());
                raw.into_draft(self.list_draft_attachments(&id)?)
            })
            .collect()
    }
}
