#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des comptes : aller-retour, liste, mise à jour, état, suppression.

mod support;

use mira_core::storage::Database;
use mira_vocabulary::{AccountId, AccountState, AuthMethodKind, Timestamp};
use support::accounts::{database_with_account, sample_account};

/// Un compte relu est identique au compte inséré, serveurs compris
#[test]
fn inserted_account_is_read_back_identical() {
    let db = database_with_account("acc-1");
    let loaded = db
        .read(|tx| tx.get_account(&AccountId::from_raw("acc-1")))
        .unwrap();
    assert_eq!(loaded, Some(sample_account("acc-1")));
}

/// Un compte sans serveurs explicites et OAuth2 survit à l'aller-retour
#[test]
fn account_without_servers_roundtrips() {
    let db = Database::open_in_memory().unwrap();
    let mut account = sample_account("acc-2");
    account.servers = None;
    account.auth = AuthMethodKind::OAuth2;
    db.write(|tx| tx.insert_account(&account)).unwrap();
    let loaded = db.read(|tx| tx.get_account(&account.id)).unwrap();
    assert_eq!(loaded, Some(account));
}

/// Un compte inconnu donne rien, et la liste suit l'ordre de création
#[test]
fn missing_account_is_none_and_list_is_ordered() {
    let db = Database::open_in_memory().unwrap();
    let mut late = sample_account("acc-late");
    late.created_at = Timestamp::from_millis(9_000);
    db.write(|tx| {
        tx.insert_account(&late)?;
        tx.insert_account(&sample_account("acc-early"))
    })
    .unwrap();
    assert!(
        db.read(|tx| tx.get_account(&AccountId::from_raw("nope")))
            .unwrap()
            .is_none()
    );
    let ids: Vec<String> = db
        .read(|tx| tx.list_accounts())
        .unwrap()
        .into_iter()
        .map(|account| account.id.into_string())
        .collect();
    assert_eq!(ids, ["acc-early", "acc-late"]);
}

/// La mise à jour change les champs modifiables ; un compte inconnu est refusé
#[test]
fn update_changes_fields_and_unknown_account_is_refused() {
    let db = database_with_account("acc-1");
    let mut account = sample_account("acc-1");
    account.display_name = "Alice Martin".to_string();
    account.sync.window_days = 30;
    db.write(|tx| tx.update_account(&account)).unwrap();
    assert_eq!(
        db.read(|tx| tx.get_account(&account.id)).unwrap(),
        Some(account)
    );
    let error = db
        .write(|tx| tx.update_account(&sample_account("ghost")))
        .unwrap_err();
    assert_eq!(error.code, "not_found.account");
}

/// L'état change et le marquage « en suppression » se relit
#[test]
fn state_and_deleting_flag_are_persisted() {
    let db = database_with_account("acc-1");
    let id = AccountId::from_raw("acc-1");
    let broken = AccountState::Error {
        code: "network.timeout".to_string(),
    };
    db.write(|tx| tx.set_account_state(&id, &broken)).unwrap();
    assert_eq!(
        db.read(|tx| tx.get_account(&id)).unwrap().unwrap().state,
        broken
    );
    assert!(!db.read(|tx| tx.account_is_deleting(&id)).unwrap());
    db.write(|tx| tx.mark_account_deleting(&id)).unwrap();
    assert!(db.read(|tx| tx.account_is_deleting(&id)).unwrap());
}

/// Supprimer un compte le retire ; le supprimer deux fois est une erreur
#[test]
fn delete_removes_the_account() {
    let db = database_with_account("acc-1");
    let id = AccountId::from_raw("acc-1");
    db.write(|tx| tx.delete_account(&id)).unwrap();
    assert!(db.read(|tx| tx.get_account(&id)).unwrap().is_none());
    assert!(db.write(|tx| tx.delete_account(&id)).is_err());
}
