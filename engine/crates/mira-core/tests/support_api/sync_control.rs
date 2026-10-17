//! Gestes de test sur un moteur qui synchronise : pause, attente, mot de passe changé côté serveur.

use std::time::Duration;

use mira_core::api::{Response, SyncRequest, SyncResponse, SyncScope};
use mira_vocabulary::{AccountId, AccountState, EventKind};

use super::fixture::{FOLDERS_PASSWORD, FOLDERS_USER, Fixture};
use super::queries::accounts;
use super::sync_calls::{sync_call, wait_for_events};

/// Laisse au moteur le temps de réagir à un courrier déposé, pour constater qu'il ne bouge pas
pub async fn settle() {
    tokio::time::sleep(Duration::from_millis(400)).await;
}

/// `PauseSync` du compte : l'état rendu est « en pause » et plus rien ne se synchronise
pub async fn pause_sync_of(f: &Fixture, account: &AccountId) {
    let request = SyncRequest::PauseSync {
        account: account.clone(),
    };
    let Response::Sync(SyncResponse::Paused(status)) = sync_call(&f.engine, request).await else {
        panic!("réponse")
    };
    assert_eq!(status.state, AccountState::Paused);
    assert!(!status.syncing);
}

/// Change le mot de passe côté serveur, coupe les connexions et attend « authentification requise »
pub async fn reject_password(f: &Fixture, account: &AccountId) {
    f.credentials.add_password(FOLDERS_USER, "changé-ailleurs");
    f.imap.drop_all_connections();
    let required = EventKind::AccountStateChanged {
        account: account.clone(),
        state: AccountState::AuthRequired,
    };
    wait_for_events(&f.engine, |kinds| kinds.contains(&required)).await;
}

/// Requête `StartSync` visant ce compte
pub fn account_scope(account: &AccountId) -> SyncRequest {
    let account = account.clone();
    SyncRequest::StartSync {
        scope: SyncScope::Account { account },
    }
}

/// Jours de plage de synchronisation enregistrés pour le compte
pub async fn window_days(f: &Fixture, account: &AccountId) -> u32 {
    let all = accounts(&f.engine).await;
    all.into_iter()
        .find(|a| a.id == *account)
        .unwrap()
        .sync
        .window_days
}

/// Remet côté serveur le mot de passe que le trousseau du moteur connaît
pub fn repair_password(f: &Fixture) {
    f.credentials.add_password(FOLDERS_USER, FOLDERS_PASSWORD);
}
