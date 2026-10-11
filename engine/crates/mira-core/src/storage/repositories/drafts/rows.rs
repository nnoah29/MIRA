//! Conversion entre une ligne de la table `drafts` et un `Draft`.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{
    AccountId, Draft, DraftAttachment, DraftId, ErrorCategory, ErrorContext, IdentityId, MessageId,
    MiraError, Timestamp,
};

use crate::storage::json_columns::{from_json, to_json};

pub(super) const COLUMNS: &str = "id, account_id, identity_id, to_json, cc_json, bcc_json, subject, body_text, body_html, in_reply_to, state_json, updated_at";

/// Ligne brute de la table `drafts`, dans l'ordre de `COLUMNS`.
pub(super) struct RawDraft {
    pub(super) id: String,
    account: String,
    identity: String,
    to_json: String,
    cc_json: String,
    bcc_json: String,
    subject: String,
    body_text: String,
    body_html: Option<String>,
    in_reply_to: Option<String>,
    state_json: String,
    updated_at: i64,
}

impl RawDraft {
    /// Lit une ligne sélectionnée avec `COLUMNS`
    pub(super) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            account: row.get(1)?,
            identity: row.get(2)?,
            to_json: row.get(3)?,
            cc_json: row.get(4)?,
            bcc_json: row.get(5)?,
            subject: row.get(6)?,
            body_text: row.get(7)?,
            body_html: row.get(8)?,
            in_reply_to: row.get(9)?,
            state_json: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }

    /// Reconstruit le brouillon du domaine, avec les métadonnées de ses pièces jointes
    pub(super) fn into_draft(self, attachments: Vec<DraftAttachment>) -> Result<Draft, MiraError> {
        Ok(Draft {
            id: DraftId::from_raw(self.id),
            account: AccountId::from_raw(self.account),
            identity: IdentityId::from_raw(self.identity),
            to: from_json(&self.to_json)?,
            cc: from_json(&self.cc_json)?,
            bcc: from_json(&self.bcc_json)?,
            subject: self.subject,
            body_text: self.body_text,
            body_html: self.body_html,
            attachments,
            in_reply_to: self.in_reply_to.map(MessageId::from_raw),
            state: from_json(&self.state_json)?,
            updated_at: Timestamp::from_millis(self.updated_at),
        })
    }
}

/// Colonnes JSON d'un brouillon, sérialisées avant l'écriture.
pub(super) struct DraftJson {
    pub(super) to: String,
    pub(super) cc: String,
    pub(super) bcc: String,
    pub(super) state: String,
}

impl DraftJson {
    /// Sérialise les destinataires et l'état du brouillon
    pub(super) fn of(draft: &Draft) -> Result<Self, MiraError> {
        Ok(Self {
            to: to_json(&draft.to)?,
            cc: to_json(&draft.cc)?,
            bcc: to_json(&draft.bcc)?,
            state: to_json(&draft.state)?,
        })
    }
}

/// Erreur « brouillon inconnu » portant l'identifiant visé
pub(super) fn draft_not_found(id: &DraftId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::DRAFT,
        "brouillon inconnu",
    )
    .with_context(ErrorContext {
        diagnostic_id: Some(id.to_string()),
        ..ErrorContext::default()
    })
}
