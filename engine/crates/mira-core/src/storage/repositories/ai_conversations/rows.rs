//! Lignes de `ai_conversations` : colonnes de métadonnées lues dans l'ordre et conversion vers `AiConversationMeta`.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{AiConversationMeta, ErrorCategory, MiraError, Timestamp};
use rusqlite::Row;

pub(super) const COLUMNS: &str = "id, title, created_at, updated_at, payload_version";

/// Lit les métadonnées d'une ligne dans l'ordre de `COLUMNS`
pub(super) fn meta_from_row(row: &Row<'_>) -> rusqlite::Result<AiConversationMeta> {
    Ok(AiConversationMeta {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: Timestamp::from_millis(row.get(2)?),
        updated_at: Timestamp::from_millis(row.get(3)?),
        payload_version: row.get(4)?,
    })
}

/// Erreur d'une conversation inconnue, sans aucun contenu
pub(super) fn unknown_conversation() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::AI_CONVERSATION,
        "conversation d'IA introuvable",
    )
}
