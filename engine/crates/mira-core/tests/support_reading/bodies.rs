//! Messages avec ou sans corps local, et pièces jointes de test.

use mira_core::storage::Database;
use mira_vocabulary::{Attachment, AttachmentId, CacheState, ContentState, MessageId};

use super::views::{database_with_many_boxes, put_custom, put_message};
use crate::support_mirror::fixtures::sample_message;

/// Base avec `full` (texte, HTML, source), `plain` (texte seul), `headers` (en-têtes seuls) et `stale` (corps annoncé mais cache vide)
pub fn database_with_bodies() -> Database {
    let db = database_with_many_boxes();
    for (id, at) in [("full", 1_000), ("stale", 3_000), ("plain", 4_000)] {
        let mut message = sample_message(id, "acc-1");
        message.content = ContentState::BodyPresent;
        put_custom(&db, &message, at, &["box-1"]);
    }
    put_message(&db, "headers", "acc-1", 2_000, &["box-1"]);
    db.write(|tx| {
        let (full, plain) = (MessageId::from_raw("full"), MessageId::from_raw("plain"));
        tx.store_parsed(&full, "Bonjour Alice", Some("<p>Bonjour Alice</p>"))?;
        tx.store_raw_source(&full, b"Subject: Sujet full\r\n\r\nBonjour Alice\r\n")?;
        tx.store_parsed(&plain, "Texte seul", None)
    })
    .unwrap();
    db
}

/// Pièce jointe PDF de test d'un message
pub fn attachment(id: &str, message: &str, filename: &str) -> Attachment {
    Attachment {
        id: AttachmentId::from_raw(id),
        message: MessageId::from_raw(message),
        filename: Some(filename.to_string()),
        mime_type: "application/pdf".to_string(),
        size: 4_096,
        content_id: None,
        inline: false,
        cache: CacheState::NotDownloaded,
    }
}
