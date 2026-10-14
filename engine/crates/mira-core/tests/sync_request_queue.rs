#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! File des demandes d'un compte : fusion (invariant 24), accusés de fin, échec groupé, portées.

use std::time::Duration;

use mira_core::sync::{RequestScope, SyncRequests};
use mira_vocabulary::error::codes::network;
use mira_vocabulary::{ErrorCategory, MailboxId, MiraError, OperationId};
use tokio::sync::oneshot;

/// Erreur réseau de test
fn cut() -> MiraError {
    MiraError::new(ErrorCategory::Network, network::CONNECTION_LOST, "coupé")
}

/// Des demandes identiques se fusionnent en une portée et ne réveillent la tâche qu'une fois
#[tokio::test]
async fn identical_requests_merge_and_wake_the_task_once() {
    let requests = SyncRequests::default();
    for _ in 0..5 {
        requests.push(RequestScope::Account, None, None);
    }
    tokio::time::timeout(Duration::from_secs(1), requests.notified())
        .await
        .unwrap();
    let second = tokio::time::timeout(Duration::from_millis(50), requests.notified()).await;
    assert!(second.is_err(), "un seul signal pour cinq demandes");
    let taken = requests.take();
    assert_eq!(taken.scopes, [RequestScope::Account]);
    assert!(requests.take().is_empty());
}

/// Chaque demandeur reçoit l'issue de la passe qui a servi sa demande
#[tokio::test]
async fn every_ticket_receives_the_outcome_of_the_pass() {
    let requests = SyncRequests::default();
    let (first, mut first_wait) = oneshot::channel();
    let (second, mut second_wait) = oneshot::channel();
    requests.push(RequestScope::Account, Some(first), None);
    requests.push(RequestScope::Account, Some(second), None);
    assert!(
        first_wait.try_recv().is_err(),
        "rien avant la fin de la passe"
    );
    requests.take().settle(&Ok(()));
    assert_eq!(first_wait.try_recv().unwrap(), Ok(()));
    assert_eq!(second_wait.try_recv().unwrap(), Ok(()));
}

/// Une rupture de session répond par l'erreur à toutes les demandes en attente et vide la file
#[tokio::test]
async fn failing_all_answers_waiting_requests_and_empties_the_queue() {
    let requests = SyncRequests::default();
    let (ticket, mut wait) = oneshot::channel();
    requests.push(RequestScope::Account, Some(ticket), None);
    requests.fail_all(&cut());
    assert_eq!(wait.try_recv().unwrap().unwrap_err().code, cut().code);
    assert!(requests.take().is_empty());
}

/// La première opération est gardée ; les portées se rangent en compte, boîtes seules et resynchronisations
#[tokio::test]
async fn first_operation_is_kept_and_scopes_are_sorted() {
    let requests = SyncRequests::default();
    let (one, two) = (MailboxId::from_raw("b-1"), MailboxId::from_raw("b-2"));
    let first = OperationId::from_raw("op-1");
    requests.push(
        RequestScope::Mailbox(one.clone()),
        None,
        Some(first.clone()),
    );
    requests.push(
        RequestScope::Resync(two.clone()),
        None,
        Some(OperationId::from_raw("op-2")),
    );
    let taken = requests.take();
    assert_eq!(taken.operation, Some(first));
    assert!(!taken.wants_account());
    assert_eq!(taken.mailboxes(), [one]);
    assert_eq!(taken.resyncs(), [two]);
}

/// Un demandeur disparu n'empêche pas de répondre aux autres
#[tokio::test]
async fn a_dropped_waiter_does_not_block_the_others() {
    let requests = SyncRequests::default();
    let (gone, gone_wait) = oneshot::channel();
    let (kept, mut kept_wait) = oneshot::channel();
    drop(gone_wait);
    requests.push(RequestScope::Account, Some(gone), None);
    requests.push(RequestScope::Account, Some(kept), None);
    requests.take().settle(&Err(cut()));
    assert!(kept_wait.try_recv().unwrap().is_err());
}
