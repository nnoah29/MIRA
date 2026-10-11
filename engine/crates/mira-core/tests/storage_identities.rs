#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des identités et des autorisations de contenu distant.

mod support;

use mira_core::storage::repositories::allowances::RemoteContentScope;
use mira_vocabulary::{AccountId, IdentityId};
use support::accounts::{database_with_account, sample_identity};

/// Invariant 11 : définir l'identité par défaut n'en laisse qu'une seule
#[test]
fn invariant_11_set_default_identity_leaves_only_one() {
    let db = database_with_account("acc-1");
    let account = AccountId::from_raw("acc-1");
    db.write(|tx| {
        tx.insert_identity(&sample_identity("main", "acc-1", true))?;
        tx.insert_identity(&sample_identity("work", "acc-1", false))?;
        tx.set_default_identity(&account, &IdentityId::from_raw("work"))
    })
    .unwrap();
    let identities = db.read(|tx| tx.list_identities(&account)).unwrap();
    let defaults: Vec<&str> = identities
        .iter()
        .filter(|identity| identity.is_default)
        .map(|identity| identity.id.as_str())
        .collect();
    assert_eq!(defaults, ["work"]);
    assert_eq!(identities[0].id.as_str(), "work");
}

/// Insérer une seconde identité par défaut remplace la première
#[test]
fn inserting_a_default_replaces_the_previous_default() {
    let db = database_with_account("acc-1");
    db.write(|tx| {
        tx.insert_identity(&sample_identity("old", "acc-1", true))?;
        tx.insert_identity(&sample_identity("new", "acc-1", true))
    })
    .unwrap();
    let old = db
        .read(|tx| tx.get_identity(&IdentityId::from_raw("old")))
        .unwrap();
    let new = db
        .read(|tx| tx.get_identity(&IdentityId::from_raw("new")))
        .unwrap();
    assert!(!old.unwrap().is_default);
    assert!(new.unwrap().is_default);
}

/// La mise à jour garde le statut par défaut ; une identité inconnue est refusée
#[test]
fn update_keeps_default_flag_and_unknown_is_refused() {
    let db = database_with_account("acc-1");
    let mut identity = sample_identity("main", "acc-1", true);
    db.write(|tx| tx.insert_identity(&identity)).unwrap();
    identity.signature = None;
    identity.is_default = false;
    db.write(|tx| tx.update_identity(&identity)).unwrap();
    let loaded = db
        .read(|tx| tx.get_identity(&identity.id))
        .unwrap()
        .unwrap();
    assert!(loaded.is_default);
    assert_eq!(loaded.signature, None);
    let ghost = sample_identity("ghost", "acc-1", false);
    assert_eq!(
        db.write(|tx| tx.update_identity(&ghost)).unwrap_err().code,
        "not_found.identity"
    );
}

/// Désigner par défaut l'identité d'un autre compte est refusé
#[test]
fn default_must_belong_to_the_account() {
    let db = database_with_account("acc-1");
    db.write(|tx| tx.insert_identity(&sample_identity("main", "acc-1", true)))
        .unwrap();
    let result = db.write(|tx| {
        tx.set_default_identity(&AccountId::from_raw("acc-2"), &IdentityId::from_raw("main"))
    });
    assert_eq!(result.unwrap_err().code, "not_found.identity");
}

/// Les autorisations sont idempotentes, séparées par portée, et l'expéditeur est insensible à la casse
#[test]
fn allowances_are_idempotent_scoped_and_case_insensitive_for_senders() {
    let db = database_with_account("acc-1");
    db.write(|tx| {
        tx.add_remote_content_allowance(RemoteContentScope::Sender, "News@Example.org")?;
        tx.add_remote_content_allowance(RemoteContentScope::Sender, "news@example.org")?;
        tx.add_remote_content_allowance(RemoteContentScope::Message, "msg-1")
    })
    .unwrap();
    let allowed = |scope, value: &str| {
        db.read(|tx| tx.is_remote_content_allowed(scope, value))
            .unwrap()
    };
    assert!(allowed(RemoteContentScope::Sender, "NEWS@example.org"));
    assert!(!allowed(RemoteContentScope::Message, "news@example.org"));
    assert!(allowed(RemoteContentScope::Message, "msg-1"));
    assert!(!allowed(RemoteContentScope::Sender, "msg-1"));
    assert_eq!(
        db.read(|tx| tx.list_remote_content_allowances())
            .unwrap()
            .len(),
        2
    );
}
