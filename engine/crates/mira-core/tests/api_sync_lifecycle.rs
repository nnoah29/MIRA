#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Cycle de vie de la synchronisation avec les comptes : pause, reprise, redémarrage, fermeture.

mod support_api;

use std::time::Duration;

use mira_core::api::{AccountRequest, Request};
use mira_vocabulary::AccountState;
use support_api::sync_calls::sync_status;
use support_api::sync_control::{pause_sync_of, settle};
use support_api::sync_reads::{subjects_in, wait_for_subjects};
use support_api::synced::synced_account;
use support_api::syncing::deliver_one;

/// `PauseSync` met le compte en pause : plus aucune connexion ni aucun message nouveau
#[tokio::test]
async fn pause_sync_stops_the_account_task_and_keeps_data_readable() {
    let (f, account) = synced_account().await;
    pause_sync_of(&f, &account).await;
    let connections = f.imap.connection_count();
    deliver_one(&f, "INBOX", "Pendant la pause");
    settle().await;
    assert_eq!(f.imap.connection_count(), connections);
    assert_eq!(subjects_in(&f.engine, &account, "Archive").await.len(), 2);
    assert!(subjects_in(&f.engine, &account, "INBOX").await.is_empty());
    let statuses = sync_status(&f.engine).await;
    assert_eq!(statuses[0].state, AccountState::Paused);
}

/// `ResumeAccount` relance la synchronisation : le courrier reçu pendant la pause arrive
#[tokio::test]
async fn resume_account_restarts_synchronization() {
    let (f, account) = synced_account().await;
    pause_sync_of(&f, &account).await;
    deliver_one(&f, "INBOX", "Reçu en pause");
    let resume = AccountRequest::ResumeAccount {
        account: account.clone(),
    };
    f.engine.execute(Request::Account(resume)).await.unwrap();
    let found = wait_for_subjects(&f.engine, &account, "INBOX", 1).await;
    assert_eq!(found, ["Reçu en pause"]);
    let statuses = sync_status(&f.engine).await;
    assert_eq!(statuses[0].state, AccountState::Connected);
}

/// Au démarrage du moteur, un compte en pause n'est pas synchronisé ; la reprise le relance
#[tokio::test]
async fn engine_start_skips_paused_accounts() {
    let (f, account) = synced_account().await;
    pause_sync_of(&f, &account).await;
    let f = f.reopen();
    let connections = f.imap.connection_count();
    deliver_one(&f, "INBOX", "Fermé");
    settle().await;
    assert_eq!(f.imap.connection_count(), connections);
    let resume = AccountRequest::ResumeAccount {
        account: account.clone(),
    };
    f.engine.execute(Request::Account(resume)).await.unwrap();
    wait_for_subjects(&f.engine, &account, "INBOX", 1).await;
}

/// Un moteur rouvert relance la tâche d'un compte actif : le courrier reçu pendant l'arrêt arrive tout seul
#[tokio::test]
async fn engine_start_resumes_synchronization_of_active_accounts() {
    let (f, account) = synced_account().await;
    let f = f.reopen();
    deliver_one(&f, "INBOX", "Pendant l'arrêt");
    let found = wait_for_subjects(&f.engine, &account, "INBOX", 1).await;
    assert_eq!(found, ["Pendant l'arrêt"]);
    let statuses = sync_status(&f.engine).await;
    assert!(statuses[0].last_synced_at.is_some());
}

/// La fermeture avec délai arrête les tâches : plus de connexion ouverte, plus de nouvelle tentative
#[tokio::test]
async fn shutdown_within_stops_every_background_task() {
    let (f, _account) = synced_account().await;
    let on_time = f.engine.shutdown_within(Duration::from_secs(5)).await;
    assert!(on_time.unwrap());
    settle().await;
    assert_eq!(f.imap.active_connections(), 0);
}
