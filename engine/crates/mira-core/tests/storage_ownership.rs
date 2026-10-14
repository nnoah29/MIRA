#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Propriété des données (invariant 10) : un message n'appartient qu'à un compte, un compte qu'à un profil.

mod support;
mod support_mirror;

use mira_vocabulary::{AccountId, MessageId, ProfileId, Timestamp};
use support::accounts::{database_with_account, sample_account};
use support_mirror::fixtures::sample_message;

/// Invariant 10 : un message sans compte connu est refusé, un même message ne se range pas sous deux comptes, un compte garde un seul profil
#[test]
fn invariant_10_message_belongs_to_one_account_account_to_one_profile() {
    let db = database_with_account("acc-1");
    db.write(|tx| tx.insert_account(&sample_account("acc-2")))
        .unwrap();
    let orphan = sample_message("msg-orphan", "acc-ghost");
    let refused = db.write(|tx| tx.insert_message(&orphan, Timestamp::from_millis(1)));
    assert!(refused.is_err(), "message d'un compte inconnu refusé");
    let owned = sample_message("msg-1", "acc-1");
    db.write(|tx| tx.insert_message(&owned, Timestamp::from_millis(1)))
        .unwrap();
    let twice = sample_message("msg-1", "acc-2");
    let duplicated = db.write(|tx| tx.insert_message(&twice, Timestamp::from_millis(2)));
    assert!(duplicated.is_err(), "un message ne change pas de compte");
    let stored = db.read(|tx| tx.get_message(&owned.id)).unwrap().unwrap();
    assert_eq!(stored.account, AccountId::from_raw("acc-1"));
    assert_eq!(db.read(|tx| tx.table_row_count("messages")).unwrap(), 1);
    let mut other_profile = sample_account("acc-1");
    other_profile.profile = ProfileId::from_raw("autre");
    let reinserted = db.write(|tx| tx.insert_account(&other_profile));
    assert!(reinserted.is_err(), "un compte n'est pas inscrit deux fois");
    let account = db.read(|tx| tx.get_account(&other_profile.id)).unwrap();
    assert_eq!(account.unwrap().profile, ProfileId::from_raw("default"));
}

/// La suppression d'un compte emporte ses messages et laisse ceux de l'autre compte
#[test]
fn deleting_an_account_removes_its_messages_only() {
    let db = database_with_account("acc-1");
    db.write(|tx| tx.insert_account(&sample_account("acc-2")))
        .unwrap();
    let at = Timestamp::from_millis(1);
    db.write(|tx| {
        tx.insert_message(&sample_message("msg-1", "acc-1"), at)?;
        tx.insert_message(&sample_message("msg-2", "acc-2"), at)?;
        tx.delete_account(&AccountId::from_raw("acc-1"))
    })
    .unwrap();
    let first = db.read(|tx| tx.get_message(&MessageId::from_raw("msg-1")));
    assert!(first.unwrap().is_none());
    let second = db.read(|tx| tx.get_message(&MessageId::from_raw("msg-2")));
    assert!(second.unwrap().is_some());
}
