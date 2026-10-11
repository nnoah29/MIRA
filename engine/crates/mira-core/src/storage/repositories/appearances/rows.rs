//! Conversion entre une ligne de la table `appearances` et une `Appearance`.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{Appearance, ErrorCategory, ErrorContext, MailboxId, MessageId, MiraError};

pub(super) const COLUMNS: &str = "message_id, mailbox_id, uid, deleted, needs_revalidation";

/// Lit une ligne sélectionnée avec `COLUMNS` ; un UID négatif (en attente de revalidation) est rendu positif
pub(super) fn appearance_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Appearance> {
    let uid: i64 = row.get(2)?;
    Ok(Appearance {
        message: MessageId::from_raw(row.get::<_, String>(0)?),
        mailbox: MailboxId::from_raw(row.get::<_, String>(1)?),
        uid: u32::try_from(uid.unsigned_abs()).unwrap_or(0),
        deleted: row.get(3)?,
        needs_revalidation: row.get(4)?,
    })
}

/// Erreur « apparition inconnue » : le couple (message, boîte) n'existe pas
pub(super) fn appearance_not_found(message: &MessageId, mailbox: &MailboxId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::MESSAGE,
        "apparition inconnue",
    )
    .with_context(ErrorContext {
        message: Some(message.clone()),
        mailbox: Some(mailbox.clone()),
        ..ErrorContext::default()
    })
}
