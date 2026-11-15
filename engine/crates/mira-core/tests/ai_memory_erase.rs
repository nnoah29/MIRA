#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Oublier un souvenir n'en laisse aucune trace lisible dans le journal WAL du fichier de base.

mod support_ai_memory;

use mira_vocabulary::{MemoryKind, MemorySource};
use support_ai_memory::setup_in;

/// Contenu du journal WAL du fichier de base, vide s'il n'existe plus
fn wal_bytes(dir: &std::path::Path) -> Vec<u8> {
    std::fs::read(dir.join("mira.sqlite3-wal")).unwrap_or_default()
}

/// Après `forget`, le texte du souvenir n'est plus dans le journal
#[tokio::test]
async fn forget_leaves_no_trace_in_the_wal() {
    let dir = tempfile::tempdir().unwrap();
    let s = setup_in(dir.path(), "m1");
    let kept = s
        .store
        .remember("Zanzibar-chat-4242", MemoryKind::Fact, MemorySource::User)
        .await
        .unwrap();
    s.store.forget(&kept.id).await.unwrap();
    let needle = b"Zanzibar-chat-4242";
    assert!(
        !wal_bytes(dir.path())
            .windows(needle.len())
            .any(|w| w == needle)
    );
}

/// Après `forget_all`, rien ne reste non plus
#[tokio::test]
async fn forget_all_leaves_no_trace_in_the_wal() {
    let dir = tempfile::tempdir().unwrap();
    let s = setup_in(dir.path(), "m1");
    s.store
        .remember("Zanzibar-chien-4343", MemoryKind::Fact, MemorySource::User)
        .await
        .unwrap();
    s.store.forget_all().await.unwrap();
    let needle = b"Zanzibar-chien-4343";
    assert!(
        !wal_bytes(dir.path())
            .windows(needle.len())
            .any(|w| w == needle)
    );
}
