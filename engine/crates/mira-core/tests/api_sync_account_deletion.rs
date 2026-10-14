#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Suppression d'un compte pendant que la synchronisation tourne (S14) et reprise d'une suppression interrompue.

mod support_api;

use std::time::Duration;

use mira_core::api::{AccountRequest, Request, Response};
use mira_vocabulary::{AccountId, AuthMethodKind, EventKind, SecretKind};
use support_api::fixture::{Fixture, LABELS_PASSWORD, LABELS_USER};
use support_api::queries::{accounts, event_kinds};
use support_api::requests::{add_account, draft};
use support_api::sync_calls::{finished_passes, wait_for_events};
use support_api::sync_reads::wait_for_subjects;
use support_api::synced::synced_account;
use support_api::syncing::mail;

/// Ajoute le compte `labels` au moteur déjà synchronisé et attend sa première passe
async fn add_second_account(f: &Fixture) -> AccountId {
    let labels = draft(LABELS_USER, AuthMethodKind::AppPassword);
    let second = add_account(&f.engine, labels, LABELS_PASSWORD, "k-second").await;
    wait_for_events(&f.engine, |k| finished_passes(k) >= 2).await;
    second.id
}

/// S14 pendant une synchro : la tâche s'arrête avant la cascade, le serveur n'est pas touché, l'autre compte continue
#[tokio::test]
async fn scenario_s14_delete_interrupts_sync_and_keeps_other_account_running() {
    let (f, doomed) = synced_account().await;
    let kept = add_second_account(&f).await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    let log_before = f.store.command_log();
    let request = Request::Account(AccountRequest::DeleteAccount {
        account: doomed.clone(),
    });
    assert_eq!(f.engine.execute(request).await.unwrap(), Response::Done);
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(f.store.command_log(), log_before, "rien envoyé au serveur");
    let kinds = event_kinds(&f.engine).await;
    let deleted = EventKind::AccountDeleted {
        account: doomed.clone(),
    };
    let at = kinds.iter().position(|k| *k == deleted).unwrap();
    let late_sync =
        |k: &EventKind| matches!(k, EventKind::SyncStarted { account, .. } if *account == doomed);
    assert!(
        !kinds[at..].iter().any(late_sync),
        "aucune synchro après la suppression"
    );
    f.store
        .deliver(LABELS_USER, "INBOX", &mail("Pour le second", "s2@x"), &[]);
    wait_for_subjects(&f.engine, &kept, "INBOX", 1).await;
    assert!(f.stored_secret(&doomed, SecretKind::Password).is_none());
}

/// S14 : une suppression interrompue (compte marqué « en suppression ») est achevée à l'ouverture suivante
#[tokio::test]
async fn scenario_s14_interrupted_deletion_is_completed_at_startup() {
    let (f, doomed) = synced_account().await;
    let kept = add_second_account(&f).await;
    assert!(f.stored_secret(&doomed, SecretKind::Password).is_some());
    let f = f.reopen_after_crash_in_deletion(&doomed);
    let remaining: Vec<AccountId> = accounts(&f.engine)
        .await
        .into_iter()
        .map(|a| a.id)
        .collect();
    assert_eq!(remaining, std::slice::from_ref(&kept));
    assert!(f.stored_secret(&doomed, SecretKind::Password).is_none());
    assert!(f.stored_secret(&kept, SecretKind::Password).is_some());
    let kinds = event_kinds(&f.engine).await;
    let deleted = EventKind::AccountDeleted { account: doomed };
    assert_eq!(kinds.iter().filter(|k| **k == deleted).count(), 1);
}

/// Rouvrir le moteur une seconde fois n'émet plus rien : la reprise d'une suppression est idempotente
#[tokio::test]
async fn finishing_a_deletion_twice_changes_nothing() {
    let (f, doomed) = synced_account().await;
    let f = f.reopen_after_crash_in_deletion(&doomed).reopen();
    let kinds = event_kinds(&f.engine).await;
    let deleted = |k: &&EventKind| matches!(k, EventKind::AccountDeleted { .. });
    assert_eq!(kinds.iter().filter(deleted).count(), 1);
    assert!(accounts(&f.engine).await.is_empty());
}
