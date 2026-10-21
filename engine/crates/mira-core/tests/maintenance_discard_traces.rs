#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Abandon du miroir d'un compte (D-18) : ses mots ne survivent ni dans l'index de recherche ni dans le journal WAL.

mod support;
mod support_mirror;

use std::fs;
use std::path::Path;

use mira_core::maintenance::discard_mirror_and_cache;
use mira_core::storage::{Database, Tx};
use mira_vocabulary::{AccountId, MailboxRole, MessageId, MiraError, Timestamp};
use support::accounts::sample_account;
use support_mirror::fixtures::{appearance, sample_mailbox, sample_message};

/// Dit si un fichier du dossier contient ces octets
fn directory_contains(folder: &Path, needle: &str) -> bool {
    fs::read_dir(folder).unwrap().any(|entry| {
        let bytes = fs::read(entry.unwrap().path()).unwrap();
        bytes
            .windows(needle.len())
            .any(|window| window == needle.as_bytes())
    })
}

/// Crée un compte, sa boîte et trente messages indexés autour du mot donné
fn seed_account(tx: &Tx<'_>, account: &str, word: &str) -> Result<(), MiraError> {
    tx.insert_account(&sample_account(account))?;
    let mailbox = format!("box-{account}");
    let inbox = sample_mailbox(&mailbox, account, "INBOX", MailboxRole::Inbox);
    tx.upsert_mailbox(&inbox)?;
    for index in 1..=30_u32 {
        let id = format!("{account}-m{index}");
        let date = Timestamp::from_millis(i64::from(index) * 1_000);
        tx.insert_message(&sample_message(&id, account), date)?;
        tx.add_appearance(&appearance(&id, &mailbox, index))?;
        let address = format!("{word}.{index}@{word}.test");
        let subject = format!("{word} sujet");
        tx.index_message(&MessageId::from_raw(id), &subject, &address, "bergamote")?;
    }
    Ok(())
}

/// Jeter le miroir par l'opération de maintenance efface les mots du compte des fichiers, pas ceux de l'autre
#[test]
fn regression_discard_mirror_and_cache_erases_traces_from_the_files() {
    let folder = tempfile::tempdir().unwrap();
    let db = Database::open(&folder.path().join("mira.sqlite3")).unwrap();
    db.write(|tx| {
        seed_account(tx, "acc-1", "zirconium")?;
        seed_account(tx, "acc-2", "nenuphar")
    })
    .unwrap();
    assert!(directory_contains(folder.path(), "zirconium"));
    let cache = folder.path().join("cache");
    discard_mirror_and_cache(&db, &AccountId::from_raw("acc-1"), &cache).unwrap();
    assert!(!directory_contains(folder.path(), "zirconium"), "mot gardé");
    assert!(
        directory_contains(folder.path(), "nenuphar"),
        "autre compte"
    );
}
