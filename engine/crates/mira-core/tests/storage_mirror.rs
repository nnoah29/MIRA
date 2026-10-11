#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Abandon du miroir (invariant 16), statistiques et intégrité.

mod support;
mod support_mirror;

use mira_vocabulary::{AccountId, MailboxId, MessageId, ThreadId};
use support::accounts::sample_identity;
use support::intentions::{sample_action, sample_draft};
use support_mirror::seeded::database_with_messages;

/// Remplit tout le miroir d'acc-1 (source, cache, index, fil, correspondance) et des intentions
fn fill_everything(db: &mira_core::storage::Database) {
    let acc = AccountId::from_raw("acc-1");
    let msg = MessageId::from_raw("msg-1");
    db.write(|tx| {
        tx.store_raw_source(&msg, b"raw")?;
        tx.store_parsed(&msg, "texte", None)?;
        tx.index_message(&msg, "sujet", "bob@example.org", "corps")?;
        let thread = tx.upsert_thread(&acc, "root", "S", &ThreadId::from_raw("t-1"))?;
        tx.set_thread(&msg, Some(&thread))?;
        tx.map_server_id(
            &acc,
            mira_core::storage::repositories::server_ids::ServerIdKind::Stable,
            "s-1",
            &msg,
        )?;
        tx.insert_identity(&sample_identity("main", "acc-1", true))?;
        tx.insert_draft(&sample_draft("d-1", "acc-1", 10))?;
        tx.enqueue_action(&sample_action("a-1", "acc-1", "k-1", 0))
            .map(|_| ())
    })
    .unwrap();
}

/// Invariant 16 : jeter le miroir vide le miroir du compte, pas sa configuration ni ses intentions
#[test]
fn invariant_16_discard_mirror_keeps_config_and_intentions() {
    let db = database_with_messages(2);
    fill_everything(&db);
    db.write(|tx| tx.discard_account_mirror(&AccountId::from_raw("acc-1")))
        .unwrap();
    db.read(|tx| {
        for table in [
            "messages",
            "mailboxes",
            "appearances",
            "raw_sources",
            "parsed_cache",
            "threads",
            "server_ids",
            "messages_fts",
        ] {
            let count = tx.table_row_count(table).unwrap();
            let expected = i64::from(table == "mailboxes");
            assert_eq!(count, expected, "table {table}");
        }
        assert_eq!(tx.table_row_count("accounts").unwrap(), 2);
        assert_eq!(tx.table_row_count("identities").unwrap(), 1);
        assert_eq!(tx.table_row_count("drafts").unwrap(), 1);
        assert_eq!(tx.table_row_count("actions").unwrap(), 1);
        Ok(())
    })
    .unwrap();
}

/// Jeter le miroir d'un compte épargne les messages et boîtes d'un autre compte
#[test]
fn discard_leaves_other_accounts_untouched() {
    let db = database_with_messages(2);
    fill_everything(&db);
    db.write(|tx| tx.discard_account_mirror(&AccountId::from_raw("acc-2")))
        .unwrap();
    let mailbox = MailboxId::from_raw("box-1");
    let kept = db.read(|tx| tx.get_mailbox(&mailbox)).unwrap();
    assert!(kept.is_some());
    let stats = db.read(|tx| tx.storage_stats()).unwrap();
    assert_eq!(
        (stats.messages, stats.mailboxes, stats.raw_sources),
        (2, 1, 1)
    );
}

/// Les statistiques comptent les volumes et le contrôle d'intégrité passe
#[test]
fn stats_count_volumes_and_integrity_check_passes() {
    let db = database_with_messages(3);
    fill_everything(&db);
    let stats = db.read(|tx| tx.storage_stats()).unwrap();
    assert_eq!(
        (stats.messages, stats.appearances, stats.threads),
        (3, 3, 1)
    );
    assert_eq!((stats.raw_sources, stats.raw_bytes), (1, 3));
    assert_eq!((stats.attachments, stats.attachments_cached), (0, 0));
    assert!(db.read(|tx| tx.integrity_check()).unwrap());
}
