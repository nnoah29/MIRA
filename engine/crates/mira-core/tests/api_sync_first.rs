#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Première synchronisation lancée par l'ajout d'un compte (S4) : réception d'abord, progression, état consultable.

mod support_api;

use mira_vocabulary::{AccountState, EventKind, SyncStep};
use support_api::fixture::{FOLDERS_USER, start};
use support_api::sync_calls::{finished_passes, sync_status, wait_for_events};
use support_api::sync_events::header_progress;
use support_api::sync_reads::{mailbox_named, subjects_in};
use support_api::syncing::{deliver_many, mail, start_syncing};

/// S4 : `AddAccount` lance la première synchro ; la réception vient d'abord, avec progression, sans « nouveau message »
#[tokio::test]
async fn scenario_s4_add_account_runs_first_sync_inbox_first_with_progress() {
    let f = start_syncing().await;
    f.store.create_mailbox(FOLDERS_USER, "Sent", Some("\\Sent"));
    f.store
        .deliver(FOLDERS_USER, "Sent", &mail("Envoyé", "s1@x"), &["\\Seen"]);
    deliver_many(&f, "INBOX", "Reçu", 3);
    let account = f.add_folders_account("k-s4").await;
    let kinds = wait_for_events(&f.engine, |k| finished_passes(k) >= 1).await;
    let added = kinds
        .iter()
        .position(|k| matches!(k, EventKind::AccountAdded { .. }));
    let started = kinds
        .iter()
        .position(|k| matches!(k, EventKind::SyncStarted { .. }));
    assert!(added.unwrap() < started.unwrap(), "la synchro suit l'ajout");
    let headers = header_progress(&kinds);
    assert_eq!(headers.first(), Some(&(0, 3)), "la réception vient d'abord");
    assert!(headers.contains(&(3, 3)) && headers.contains(&(1, 1)));
    let has_step = |wanted: SyncStep| {
        let step =
            |k: &EventKind| matches!(k, EventKind::SyncProgress { step, .. } if *step == wanted);
        kinds.iter().any(step)
    };
    assert!(has_step(SyncStep::Mailboxes) && has_step(SyncStep::Bodies));
    let announced = |k: &&EventKind| matches!(k, EventKind::NewMessageReceived { .. });
    assert_eq!(kinds.iter().filter(announced).count(), 0);
    assert_eq!(subjects_in(&f.engine, &account, "INBOX").await.len(), 3);
    assert_eq!(subjects_in(&f.engine, &account, "Sent").await, ["Envoyé"]);
}

/// Les corps récents de la réception sont préchargés par la première synchro, et la boîte porte son repère
#[tokio::test]
async fn first_sync_prefetches_inbox_bodies_and_counts_messages() {
    let f = start_syncing().await;
    deliver_many(&f, "INBOX", "Courrier", 2);
    let account = f.add_folders_account("k-bodies").await;
    wait_for_events(&f.engine, |k| {
        k.iter()
            .filter(|e| matches!(e, EventKind::BodyAvailable { .. }))
            .count()
            >= 2
    })
    .await;
    let inbox = mailbox_named(&f.engine, &account, "INBOX").await;
    assert_eq!((inbox.total, inbox.unread), (2, 2));
    assert!(
        inbox.marker.is_some(),
        "repère enregistré après les données"
    );
}

/// `GetSyncStatus` rend l'état de chaque compte : connecté, au repos, dernière synchro réussie, sans erreur
#[tokio::test]
async fn sync_status_reports_connected_idle_account_after_first_sync() {
    let f = start_syncing().await;
    deliver_many(&f, "INBOX", "Statut", 1);
    let account = f.add_folders_account("k-status").await;
    wait_for_events(&f.engine, |k| finished_passes(k) >= 1).await;
    let statuses = sync_status(&f.engine).await;
    assert_eq!(statuses.len(), 1);
    let status = &statuses[0];
    assert_eq!(status.account, account);
    assert_eq!(status.state, AccountState::Connected);
    assert!(!status.syncing && status.last_error.is_none());
    assert!(status.last_synced_at.is_some());
    assert!(status.mailboxes_in_progress.is_empty());
    assert_eq!(status.pending_actions, 0);
}

/// Sans synchronisation en arrière-plan, l'ajout d'un compte ne contacte pas le serveur au-delà du test de connexion
#[tokio::test]
async fn add_account_without_background_sync_stays_idle() {
    let f = start().await;
    deliver_many(&f, "INBOX", "Muet", 2);
    let account = f.add_folders_account("k-off").await;
    let after_add = f.imap.connection_count();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert_eq!(f.imap.connection_count(), after_add);
    let statuses = sync_status(&f.engine).await;
    assert_eq!(statuses[0].account, account);
    assert!(statuses[0].last_synced_at.is_none() && !statuses[0].syncing);
}

/// Aucun secret dans les événements ni dans l'état de synchronisation (invariant 15)
#[tokio::test]
async fn sync_events_and_status_never_contain_the_password() {
    let f = start_syncing().await;
    deliver_many(&f, "INBOX", "Discret", 2);
    f.add_folders_account("k-secret").await;
    let kinds = wait_for_events(&f.engine, |k| finished_passes(k) >= 1).await;
    let statuses = sync_status(&f.engine).await;
    let events = serde_json::to_string(&kinds).unwrap();
    let states = serde_json::to_string(&statuses).unwrap();
    let shown = format!("{events}{states}");
    assert!(!shown.contains(support_api::fixture::FOLDERS_PASSWORD));
    assert!(!shown.contains("SECRET"));
}
