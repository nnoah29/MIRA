#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Synchronisation en temps réel par le moteur : nouveau message (S5), coupure et retour en ligne, authentification.

mod support_api;

use mira_core::operations::OperationStatus;
use mira_vocabulary::{AccountState, EventKind};
use support_api::sync_calls::{operation_outcome, start_operation, sync_status, wait_for_events};
use support_api::sync_control::{account_scope, reject_password, repair_password};
use support_api::sync_events::count_kinds;
use support_api::sync_reads::{subjects_in, wait_for_subjects};
use support_api::synced::synced_account;
use support_api::syncing::deliver_one;

/// S5 : un message qui arrive pendant que le moteur tourne est lu sur la seconde connexion et annoncé
#[tokio::test]
async fn scenario_s5_engine_idle_new_message_emits_event() {
    let (f, account) = synced_account().await;
    let before = wait_for_events(&f.engine, |_| true).await.len();
    deliver_one(&f, "INBOX", "Arrivé en direct");
    let kinds = wait_for_events(&f.engine, |k| {
        count_kinds(&k[before..], |e| {
            matches!(e, EventKind::NewMessageReceived { .. })
        }) == 1
    })
    .await;
    let announced = kinds.iter().find_map(|k| match k {
        EventKind::NewMessageReceived { account, .. } => Some(account.clone()),
        _ => None,
    });
    assert_eq!(announced, Some(account.clone()));
    let found = wait_for_subjects(&f.engine, &account, "INBOX", 1).await;
    assert_eq!(found, ["Arrivé en direct"]);
}

/// Après une coupure réseau, le compte passe hors ligne puis revient en ligne et rattrape le courrier manqué
#[tokio::test]
async fn network_cut_goes_offline_then_back_online_and_catches_up() {
    let (f, account) = synced_account().await;
    let back = |k: &[EventKind]| count_kinds(k, |e| matches!(e, EventKind::BackOnline { .. }));
    let back_before = back(&wait_for_events(&f.engine, |_| true).await);
    f.imap.drop_all_connections();
    deliver_one(&f, "INBOX", "Pendant la coupure");
    let kinds = wait_for_events(&f.engine, |k| back(k) > back_before).await;
    assert!(kinds.contains(&EventKind::WentOffline {
        account: account.clone()
    }));
    let found = wait_for_subjects(&f.engine, &account, "INBOX", 1).await;
    assert_eq!(found, ["Pendant la coupure"]);
    assert_eq!(
        sync_status(&f.engine).await[0].state,
        AccountState::Connected
    );
}

/// Un mot de passe devenu invalide met le compte en « authentification requise » et fait échouer la synchro demandée
#[tokio::test]
async fn rejected_password_requires_authentication_and_fails_requested_sync() {
    let (f, account) = synced_account().await;
    reject_password(&f, &account).await;
    let status = sync_status(&f.engine).await.remove(0);
    assert_eq!(
        status.last_error.as_deref(),
        Some("authentication.rejected")
    );
    let failed = start_operation(&f.engine, account_scope(&account)).await;
    let outcome = operation_outcome(&f.engine, &failed).await;
    let code = "authentication.rejected".to_string();
    assert_eq!(outcome, OperationStatus::Failed { code });
    assert_eq!(
        sync_status(&f.engine).await[0].state,
        AccountState::AuthRequired
    );
}

/// Une demande après réparation du mot de passe relance la tâche : le compte revient et rattrape le courrier
#[tokio::test]
async fn sync_request_after_repair_restarts_the_account() {
    let (f, account) = synced_account().await;
    reject_password(&f, &account).await;
    repair_password(&f);
    deliver_one(&f, "INBOX", "Après réparation");
    let repaired = start_operation(&f.engine, account_scope(&account)).await;
    let outcome = operation_outcome(&f.engine, &repaired).await;
    assert_eq!(outcome, OperationStatus::Finished);
    assert_eq!(subjects_in(&f.engine, &account, "INBOX").await.len(), 1);
    assert_eq!(
        sync_status(&f.engine).await[0].state,
        AccountState::Connected
    );
}
