#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Demandes de synchronisation par la façade : portées, opération longue, fusion des demandes (invariant 24).

mod support_api;

use mira_core::api::{Request, SyncRequest, SyncScope};
use mira_core::operations::OperationStatus;
use mira_vocabulary::{AuthMethodKind, EventKind, MailboxId};
use support_api::fixture::{self, FOLDERS_USER, LABELS_PASSWORD, LABELS_USER};
use support_api::requests::{add_account, draft};
use support_api::sync_calls::{
    finished_passes, operation_outcome, start_operation, wait_for_events,
};
use support_api::sync_control::{account_scope, pause_sync_of};
use support_api::sync_reads::{mailbox_named, subjects_in};
use support_api::synced::synced_account;
use support_api::syncing::deliver_one;

/// `StartSync` d'un compte rend une opération qui se termine ; la passe porte l'identifiant de l'opération
#[tokio::test]
async fn start_sync_account_runs_a_pass_carrying_the_operation_id() {
    let (f, account) = synced_account().await;
    deliver_one(&f, "Archive", "Nouveau");
    let info = start_operation(&f.engine, account_scope(&account)).await;
    assert_eq!(info.kind, "sync");
    let outcome = operation_outcome(&f.engine, &info).await;
    assert_eq!(outcome, OperationStatus::Finished);
    let subjects = subjects_in(&f.engine, &account, "Archive").await;
    assert_eq!(subjects.len(), 3, "{subjects:?}");
    let kinds = wait_for_events(&f.engine, |k| finished_passes(k) >= 2).await;
    let finished = |k: &EventKind| matches!(k, EventKind::SyncFinished { operation, .. } if *operation == info.id);
    assert!(
        kinds.iter().any(finished),
        "événements de la passe demandée"
    );
}

/// `StartSync { all }` synchronise chaque compte actif du profil
#[tokio::test]
async fn start_sync_all_covers_every_active_account() {
    let (f, first) = synced_account().await;
    let labels = draft(LABELS_USER, AuthMethodKind::AppPassword);
    let second = add_account(&f.engine, labels, LABELS_PASSWORD, "k-second")
        .await
        .id;
    wait_for_events(&f.engine, |k| finished_passes(k) >= 2).await;
    deliver_one(&f, "Archive", "Pour le premier");
    f.store.create_mailbox(LABELS_USER, "Dossier", None);
    f.store.deliver(
        LABELS_USER,
        "Dossier",
        &support_api::syncing::mail("Pour le second", "p2@x"),
        &[],
    );
    let all = SyncRequest::StartSync {
        scope: SyncScope::All,
    };
    let info = start_operation(&f.engine, all).await;
    assert_eq!(
        operation_outcome(&f.engine, &info).await,
        OperationStatus::Finished
    );
    assert_eq!(subjects_in(&f.engine, &first, "Archive").await.len(), 3);
    assert_eq!(
        subjects_in(&f.engine, &second, "Dossier").await,
        ["Pour le second"]
    );
}

/// `StartSync { mailbox }` ne relève que cette boîte : les autres restent telles quelles
#[tokio::test]
async fn start_sync_mailbox_refreshes_only_that_mailbox() {
    let (f, account) = synced_account().await;
    f.store.create_mailbox(FOLDERS_USER, "Autre", None);
    deliver_one(&f, "Archive", "Dans Archive");
    deliver_one(&f, "Autre", "Dans Autre");
    let archive: MailboxId = mailbox_named(&f.engine, &account, "Archive").await.id;
    let request = SyncRequest::StartSync {
        scope: SyncScope::Mailbox { mailbox: archive },
    };
    let info = start_operation(&f.engine, request).await;
    assert_eq!(
        operation_outcome(&f.engine, &info).await,
        OperationStatus::Finished
    );
    assert_eq!(subjects_in(&f.engine, &account, "Archive").await.len(), 3);
    let others = mira_core::api::MailboxRequest::ListMailboxes { account };
    let listed = f.engine.execute(Request::Mailbox(others)).await.unwrap();
    let mira_core::api::Response::Mailboxes(boxes) = listed else {
        panic!("réponse")
    };
    assert!(
        boxes.iter().all(|m| m.name != "Autre"),
        "boîte inconnue du miroir"
    );
}

/// Invariant 24 : des demandes en rafale pendant une synchro sont fusionnées, jamais deux passes en parallèle
#[tokio::test]
async fn invariant_24_burst_of_start_sync_requests_is_merged() {
    let (f, account) = synced_account().await;
    let before = wait_for_events(&f.engine, |_| true).await.len();
    let mut operations = Vec::new();
    for _ in 0..8 {
        operations.push(start_operation(&f.engine, account_scope(&account)).await);
    }
    for info in &operations {
        let outcome = operation_outcome(&f.engine, info).await;
        assert_eq!(outcome, OperationStatus::Finished);
    }
    let kinds = wait_for_events(&f.engine, |_| true).await;
    let (mut open, mut peak, mut passes) = (0, 0, 0);
    for kind in &kinds[before..] {
        match kind {
            EventKind::SyncStarted { .. } => {
                open += 1;
                passes += 1;
            }
            EventKind::SyncFinished { .. } | EventKind::SyncFailed { .. } => open -= 1,
            _ => {}
        }
        peak = std::cmp::max(peak, open);
    }
    assert_eq!(peak, 1, "une seule passe à la fois");
    assert!((1..=3).contains(&passes), "{passes} passes pour 8 demandes");
}

/// Un compte en pause refuse `StartSync` ; `StartSync { all }` l'ignore et se termine aussitôt
#[tokio::test]
async fn start_sync_is_refused_for_paused_accounts_and_all_skips_them() {
    let (f, account) = synced_account().await;
    pause_sync_of(&f, &account).await;
    let refused = f
        .engine
        .execute(Request::Sync(account_scope(&account)))
        .await;
    assert_eq!(
        refused.unwrap_err().code,
        "incompatible_state.account_paused"
    );
    let all = SyncRequest::StartSync {
        scope: SyncScope::All,
    };
    let info = start_operation(&f.engine, all).await;
    assert_eq!(
        operation_outcome(&f.engine, &info).await,
        OperationStatus::Finished
    );
}

/// Sans synchronisation en arrière-plan, la demande est refusée avec `incompatible_state.sync_disabled`
#[tokio::test]
async fn start_sync_without_background_sync_is_refused() {
    let quiet = fixture::start().await;
    let id = quiet.add_folders_account("k-quiet").await;
    let disabled = quiet
        .engine
        .execute(Request::Sync(account_scope(&id)))
        .await;
    assert_eq!(
        disabled.unwrap_err().code,
        "incompatible_state.sync_disabled"
    );
}
