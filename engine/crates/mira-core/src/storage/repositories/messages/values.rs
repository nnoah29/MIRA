//! Valeurs d'insertion d'un message, dans l'ordre de `INSERT_COLUMNS`.

use mira_vocabulary::{MessageSummary, MiraError, Timestamp};
use rusqlite::types::Value;

use super::content_state::content_state_name;
use crate::storage::json_columns::to_json;

pub(super) const INSERT_SQL: &str = "INSERT INTO messages (id, account_id, thread_id, from_json, to_json, cc_json, bcc_json, reply_to_json, subject, date, internal_date, message_id_header, in_reply_to, references_json, unsubscribe_json, seen, flagged, answered, forwarded, size, preview, has_attachments, content_state, sort_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)";

/// Valeurs des 15 premières colonnes : identité, adresses, en-têtes et désabonnement
pub(super) fn header_values(summary: &MessageSummary) -> Result<Vec<Value>, MiraError> {
    let unsubscribe = summary.unsubscribe.as_ref().map(to_json).transpose()?;
    Ok(vec![
        Value::from(summary.id.as_str().to_string()),
        Value::from(summary.account.as_str().to_string()),
        Value::from(summary.thread.as_ref().map(|id| id.as_str().to_string())),
        Value::from(to_json(&summary.from)?),
        Value::from(to_json(&summary.to)?),
        Value::from(to_json(&summary.cc)?),
        Value::from(to_json(&summary.bcc)?),
        Value::from(to_json(&summary.reply_to)?),
        Value::from(summary.subject.clone()),
        Value::from(summary.date.map(|date| date.as_millis())),
        Value::from(summary.internal_date.map(|date| date.as_millis())),
        Value::from(summary.message_id_header.clone()),
        Value::from(summary.in_reply_to.clone()),
        Value::from(to_json(&summary.references)?),
        Value::from(unsubscribe),
    ])
}

/// Valeurs des 9 dernières colonnes : drapeaux, taille, aperçu, état du contenu et date de tri
pub(super) fn state_values(summary: &MessageSummary, sort_date: Timestamp) -> Vec<Value> {
    let flags = summary.flags;
    vec![
        Value::from(i64::from(flags.seen)),
        Value::from(i64::from(flags.flagged)),
        Value::from(i64::from(flags.answered)),
        Value::from(i64::from(flags.forwarded)),
        Value::from(i64::try_from(summary.size).unwrap_or(i64::MAX)),
        Value::from(summary.preview.clone()),
        Value::from(i64::from(summary.has_attachments)),
        Value::from(content_state_name(summary.content).to_string()),
        Value::from(sort_date.as_millis()),
    ]
}
