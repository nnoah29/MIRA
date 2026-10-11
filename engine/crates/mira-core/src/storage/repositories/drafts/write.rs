//! Écritures sur les brouillons : insertion, mise à jour, suppression.

use mira_vocabulary::{Draft, DraftId, MiraError};
use rusqlite::params;

use super::rows::{DraftJson, draft_not_found};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Insère un brouillon ; ses pièces jointes s'ajoutent séparément
    pub fn insert_draft(&self, draft: &Draft) -> Result<(), MiraError> {
        let json = DraftJson::of(draft)?;
        self.sql()
            .execute(
                "INSERT INTO drafts (id, account_id, identity_id, to_json, cc_json, bcc_json, subject, body_text, body_html, in_reply_to, state_json, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    draft.id.as_str(),
                    draft.account.as_str(),
                    draft.identity.as_str(),
                    json.to,
                    json.cc,
                    json.bcc,
                    draft.subject,
                    draft.body_text,
                    draft.body_html,
                    draft.in_reply_to.as_ref().map(|id| id.as_str()),
                    json.state,
                    draft.updated_at.as_millis(),
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Met à jour le contenu et l'état d'un brouillon (le compte et les pièces jointes restent)
    pub fn update_draft(&self, draft: &Draft) -> Result<(), MiraError> {
        let json = DraftJson::of(draft)?;
        let changed = self
            .sql()
            .execute(
                "UPDATE drafts SET identity_id = ?2, to_json = ?3, cc_json = ?4, bcc_json = ?5, subject = ?6, body_text = ?7, body_html = ?8, in_reply_to = ?9, state_json = ?10, updated_at = ?11 WHERE id = ?1",
                params![
                    draft.id.as_str(),
                    draft.identity.as_str(),
                    json.to,
                    json.cc,
                    json.bcc,
                    draft.subject,
                    draft.body_text,
                    draft.body_html,
                    draft.in_reply_to.as_ref().map(|id| id.as_str()),
                    json.state,
                    draft.updated_at.as_millis(),
                ],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(draft_not_found(&draft.id));
        }
        Ok(())
    }

    /// Supprime un brouillon et ses pièces jointes
    pub fn delete_draft(&self, id: &DraftId) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute("DELETE FROM drafts WHERE id = ?1", [id.as_str()])
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(draft_not_found(id));
        }
        Ok(())
    }
}
