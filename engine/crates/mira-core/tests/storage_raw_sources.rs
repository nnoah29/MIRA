#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Source brute compressée (D-16), cache d'analyse et statistiques de volume.

mod support;
mod support_mirror;

use mira_vocabulary::{AccountId, MessageId};
use support_mirror::seeded::database_with_messages;

/// Invariant 14 : la source brute revient identique et le stockage est plus petit que l'original
#[test]
fn invariant_14_raw_source_roundtrip_compressed() {
    let db = database_with_messages(1);
    let id = MessageId::from_raw("msg-1");
    let raw: Vec<u8> = b"Subject: test\r\n\r\nBonjour tout le monde.\r\n".repeat(500);
    db.write(|tx| tx.store_raw_source(&id, &raw)).unwrap();
    let loaded = db.read(|tx| tx.load_raw_source(&id)).unwrap();
    assert_eq!(loaded, Some(raw.clone()));
    let stats = db.read(|tx| tx.storage_stats()).unwrap();
    assert_eq!(stats.raw_bytes, raw.len() as u64);
    assert!(stats.compressed_bytes < stats.raw_bytes / 5);
}

/// Une source absente donne rien, et une source remplacée ne laisse que la dernière version
#[test]
fn missing_raw_source_is_none_and_replace_keeps_last() {
    let db = database_with_messages(1);
    let id = MessageId::from_raw("msg-1");
    assert_eq!(db.read(|tx| tx.load_raw_source(&id)).unwrap(), None);
    db.write(|tx| {
        tx.store_raw_source(&id, b"premier")?;
        tx.store_raw_source(&id, b"second")
    })
    .unwrap();
    let loaded = db.read(|tx| tx.load_raw_source(&id)).unwrap();
    assert_eq!(loaded.as_deref(), Some(&b"second"[..]));
}

/// Le cache d'analyse s'écrit, se relit et se vide par compte sans toucher la source brute
#[test]
fn parsed_cache_roundtrips_and_clears_per_account() {
    let db = database_with_messages(2);
    let (one, two) = (MessageId::from_raw("msg-1"), MessageId::from_raw("msg-2"));
    db.write(|tx| {
        tx.store_parsed(&one, "texte été", Some("<p>texte</p>"))?;
        tx.store_parsed(&two, "autre", None)?;
        tx.store_raw_source(&one, b"raw")
    })
    .unwrap();
    let loaded = db.read(|tx| tx.load_parsed(&one)).unwrap().unwrap();
    assert_eq!(loaded.text, "texte été");
    assert_eq!(loaded.html.as_deref(), Some("<p>texte</p>"));
    db.write(|tx| tx.clear_parsed_cache(&AccountId::from_raw("acc-1")))
        .unwrap();
    assert!(db.read(|tx| tx.load_parsed(&two)).unwrap().is_none());
    assert!(db.read(|tx| tx.load_raw_source(&one)).unwrap().is_some());
}
