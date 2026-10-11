#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Recherche de comptes : un compte en cours de suppression est visible mais refusé aux opérations.

use mira_core::accounts::{require_account, require_active_account};
use mira_core::storage::Database;
use mira_vocabulary::{
    Account, AccountId, AccountState, AuthMethodKind, EmailAddress, ProfileId, ProviderId,
    SyncSettings, Timestamp,
};

/// Compte minimal pour la base de test
fn sample(id: &str) -> Account {
    Account {
        id: AccountId::from_raw(id),
        profile: ProfileId::from_raw("default"),
        address: EmailAddress::parse("a@folders.test").unwrap(),
        display_name: "A".to_string(),
        provider: ProviderId::new("fake_folders"),
        auth: AuthMethodKind::Password,
        servers: None,
        sync: SyncSettings::default(),
        state: AccountState::Connected,
        created_at: Timestamp::from_millis(1),
    }
}

/// Un compte marqué en suppression est refusé par `require_active_account` mais lisible par `require_account`
#[test]
fn deleting_account_is_refused_by_active_lookup() {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| tx.insert_account(&sample("acc"))).unwrap();
    db.write(|tx| tx.mark_account_deleting(&AccountId::from_raw("acc")))
        .unwrap();
    let id = AccountId::from_raw("acc");
    let error = db.read(|tx| require_active_account(tx, &id)).unwrap_err();
    assert_eq!(error.code, "incompatible_state.account_deleting");
    assert_eq!(error.context.account, Some(id.clone()));
    assert!(db.read(|tx| require_account(tx, &id)).is_ok());
}

/// Un compte inconnu donne `not_found.account` avec son identifiant en contexte
#[test]
fn unknown_account_lookup_names_the_account() {
    let db = Database::open_in_memory().unwrap();
    let id = AccountId::from_raw("ghost");
    let error = db.read(|tx| require_account(tx, &id)).unwrap_err();
    assert_eq!(error.code, "not_found.account");
    assert_eq!(error.context.account, Some(id));
}
