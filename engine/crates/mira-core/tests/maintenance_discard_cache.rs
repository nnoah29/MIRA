#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Abandon du miroir d'un compte (D-18, AG-13) : ses fichiers de cache partent, jamais ceux d'ailleurs.

mod support;
mod support_mirror;

use std::fs;
use std::path::Path;

use mira_core::maintenance::discard_mirror_and_cache;
use mira_core::storage::Database;
use mira_vocabulary::{AccountId, Attachment, AttachmentId, CacheState, MessageId, Timestamp};
use support_mirror::fixtures::{appearance, sample_message};
use support_mirror::seeded::database_with_two_accounts;

/// Pièce jointe de test d'un message
fn attachment(id: &str, message: &str) -> Attachment {
    Attachment {
        id: AttachmentId::from_raw(id),
        message: MessageId::from_raw(message),
        filename: Some(format!("{id}.bin")),
        mime_type: "application/octet-stream".to_string(),
        size: 3,
        content_id: None,
        inline: false,
        cache: CacheState::NotDownloaded,
    }
}

/// Écrit un fichier et le déclare comme cache de la pièce jointe
fn cache_file(db: &Database, id: &str, path: &Path) {
    fs::write(path, b"abc").unwrap();
    let text = path.to_string_lossy().to_string();
    db.write(|tx| tx.set_attachment_cache_path(&AttachmentId::from_raw(id), Some(&text)))
        .unwrap();
}

/// Base à deux comptes ; acc-1 a deux pièces jointes (att-1, att-out), acc-2 une (att-2)
fn seeded() -> Database {
    let db = database_with_two_accounts();
    db.write(|tx| {
        let date = Timestamp::from_millis(1_000);
        tx.insert_message(&sample_message("msg-1", "acc-1"), date)?;
        tx.add_appearance(&appearance("msg-1", "box-1", 1))?;
        tx.insert_message(&sample_message("msg-2", "acc-2"), date)?;
        tx.add_appearance(&appearance("msg-2", "box-2", 1))?;
        tx.insert_attachment(&attachment("att-1", "msg-1"), 1)?;
        tx.insert_attachment(&attachment("att-out", "msg-1"), 2)?;
        tx.insert_attachment(&attachment("att-2", "msg-2"), 1)
    })
    .unwrap();
    db
}

/// Non-régression AG-13 : jeter le miroir efface les fichiers de cache du compte seulement, et seulement sous le cache
#[test]
fn regression_discarding_a_mirror_removes_its_attachment_cache_files() {
    let db = seeded();
    let profile = tempfile::tempdir().unwrap();
    let cache = profile.path().join("attachments");
    fs::create_dir(&cache).unwrap();
    let outside = profile.path().join("precious.bin");
    cache_file(&db, "att-1", &cache.join("att-1"));
    cache_file(&db, "att-2", &cache.join("att-2"));
    cache_file(&db, "att-out", &outside);
    discard_mirror_and_cache(&db, &AccountId::from_raw("acc-1"), &cache).unwrap();
    assert!(!cache.join("att-1").exists(), "cache du compte conservé");
    assert!(
        cache.join("att-2").exists(),
        "cache d'un autre compte perdu"
    );
    assert!(outside.exists(), "fichier hors du cache supprimé");
    let messages = db.read(|tx| tx.table_row_count("messages")).unwrap();
    assert_eq!(messages, 1);
}

/// Un chemin qui remonte par `..` hors du cache n'est jamais supprimé
#[test]
fn a_cache_path_climbing_out_of_the_cache_is_never_removed() {
    let db = seeded();
    let profile = tempfile::tempdir().unwrap();
    let cache = profile.path().join("attachments");
    fs::create_dir(&cache).unwrap();
    let outside = profile.path().join("precious.bin");
    let sneaky = cache.join("..").join("precious.bin");
    fs::write(&outside, b"abc").unwrap();
    let text = sneaky.to_string_lossy().to_string();
    db.write(|tx| tx.set_attachment_cache_path(&AttachmentId::from_raw("att-1"), Some(&text)))
        .unwrap();
    discard_mirror_and_cache(&db, &AccountId::from_raw("acc-1"), &cache).unwrap();
    assert!(outside.exists(), "le fichier hors du cache a été supprimé");
}
