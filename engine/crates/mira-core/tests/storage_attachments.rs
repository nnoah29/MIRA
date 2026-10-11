#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des pièces jointes : insertion, liste ordonnée, chemin de cache.

mod support;
mod support_mirror;

use mira_vocabulary::{Attachment, AttachmentId, CacheState, MessageId};
use support_mirror::seeded::database_with_messages;

/// Pièce jointe de test d'un message
fn attachment(id: &str, message: &str, filename: Option<&str>) -> Attachment {
    Attachment {
        id: AttachmentId::from_raw(id),
        message: MessageId::from_raw(message),
        filename: filename.map(str::to_string),
        mime_type: "application/pdf".to_string(),
        size: 4_096,
        content_id: Some(format!("<{id}@example.org>")),
        inline: false,
        cache: CacheState::NotDownloaded,
    }
}

/// Les pièces jointes d'un message sont listées dans l'ordre des parties
#[test]
fn attachments_are_listed_in_part_order() {
    let db = database_with_messages(2);
    let message = MessageId::from_raw("msg-1");
    db.write(|tx| {
        tx.insert_attachment(&attachment("att-b", "msg-1", Some("b.pdf")), 2)?;
        tx.insert_attachment(&attachment("att-a", "msg-1", None), 1)?;
        tx.insert_attachment(&attachment("att-c", "msg-2", Some("c.pdf")), 1)
    })
    .unwrap();
    let listed = db.read(|tx| tx.list_attachments(&message)).unwrap();
    let ids: Vec<&str> = listed.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids, ["att-a", "att-b"]);
    assert_eq!(listed[0], attachment("att-a", "msg-1", None));
}

/// Le chemin de cache passe l'état à « en cache » ; l'effacer le remet à « non téléchargée »
#[test]
fn cache_path_drives_cache_state() {
    let db = database_with_messages(1);
    let id = AttachmentId::from_raw("att-a");
    db.write(|tx| tx.insert_attachment(&attachment("att-a", "msg-1", Some("a.pdf")), 1))
        .unwrap();
    db.write(|tx| tx.set_attachment_cache_path(&id, Some("cache/att-a")))
        .unwrap();
    let cached = db.read(|tx| tx.get_attachment(&id)).unwrap().unwrap();
    assert_eq!(cached.cache, CacheState::Cached);
    let path = db.read(|tx| tx.attachment_cache_path(&id)).unwrap();
    assert_eq!(path.as_deref(), Some("cache/att-a"));
    db.write(|tx| tx.set_attachment_cache_path(&id, None))
        .unwrap();
    let cleared = db.read(|tx| tx.get_attachment(&id)).unwrap().unwrap();
    assert_eq!(cleared.cache, CacheState::NotDownloaded);
}

/// Une pièce jointe inconnue donne rien, et l'écriture de son cache est refusée
#[test]
fn unknown_attachment_is_none_and_cache_write_is_refused() {
    let db = database_with_messages(1);
    let id = AttachmentId::from_raw("nope");
    assert!(db.read(|tx| tx.get_attachment(&id)).unwrap().is_none());
    let refused = db.write(|tx| tx.set_attachment_cache_path(&id, Some("x")));
    assert_eq!(refused.unwrap_err().code, "not_found.attachment");
}
