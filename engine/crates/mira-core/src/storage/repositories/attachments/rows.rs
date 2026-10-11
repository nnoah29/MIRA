//! Conversion entre une ligne de la table `attachments` et une `Attachment`.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{Attachment, AttachmentId, CacheState, ErrorCategory, MessageId, MiraError};

pub(super) const COLUMNS: &str =
    "id, message_id, filename, mime_type, size, content_id, inline, cache_path";

/// Lit une ligne sélectionnée avec `COLUMNS` ; l'état du cache se déduit du chemin
pub(super) fn attachment_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Attachment> {
    let cache_path: Option<String> = row.get(7)?;
    Ok(Attachment {
        id: AttachmentId::from_raw(row.get::<_, String>(0)?),
        message: MessageId::from_raw(row.get::<_, String>(1)?),
        filename: row.get(2)?,
        mime_type: row.get(3)?,
        size: u64::try_from(row.get::<_, i64>(4)?).unwrap_or(0),
        content_id: row.get(5)?,
        inline: row.get(6)?,
        cache: match cache_path {
            Some(_) => CacheState::Cached,
            None => CacheState::NotDownloaded,
        },
    })
}

/// Erreur « pièce jointe inconnue »
pub(super) fn attachment_not_found() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::ATTACHMENT,
        "pièce jointe inconnue",
    )
}
