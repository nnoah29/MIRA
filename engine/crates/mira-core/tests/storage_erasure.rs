#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Jeter le miroir d'un compte efface aussi ses traces (index de recherche, journal WAL) sans toucher l'autre compte.

mod support;
mod support_mirror;

use std::fs;
use std::path::Path;

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

/// Base de fichier à deux comptes, le mot propre à acc-1 étant « zirconium », à acc-2 « nenuphar »
fn seeded_file_database(folder: &Path) -> Database {
    let db = Database::open(&folder.join("mira.sqlite3")).unwrap();
    db.write(|tx| {
        seed_account(tx, "acc-1", "zirconium")?;
        seed_account(tx, "acc-2", "nenuphar")
    })
    .unwrap();
    db
}

/// Après `discard_mirror_of`, les mots du compte jeté ont disparu des fichiers, ceux de l'autre compte restent
#[test]
fn discarding_a_mirror_erases_its_words_from_the_database_files() {
    let folder = tempfile::tempdir().unwrap();
    let db = seeded_file_database(folder.path());
    assert!(directory_contains(folder.path(), "zirconium"));
    db.discard_mirror_of(&AccountId::from_raw("acc-1")).unwrap();
    assert!(!directory_contains(folder.path(), "zirconium"), "mot gardé");
    assert!(
        !directory_contains(folder.path(), "acc-1-m3"),
        "message gardé"
    );
    assert!(
        directory_contains(folder.path(), "nenuphar"),
        "autre compte"
    );
}
