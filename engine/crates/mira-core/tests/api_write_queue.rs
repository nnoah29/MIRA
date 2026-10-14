#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! File d'actions par la façade : lister par état, annuler, relancer (12.4, inv. 22).

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_write;

use mira_core::api::{QueueRequest, Request};
use mira_vocabulary::{AccountId, ActionId, ActionKind, ActionState};
use support_imap::within;
use support_write::fixture::Fixture;
use support_write::mailbox_builders::move_to;

/// Requête d'annulation de la file
fn cancel(action: &ActionId) -> Request {
    Request::Queue(QueueRequest::CancelAction {
        action: action.clone(),
    })
}

/// Requête de relance de la file
fn retry(action: &ActionId) -> Request {
    Request::Queue(QueueRequest::RetryAction {
        action: action.clone(),
    })
}

/// ListActions filtre par état et garde l'ordre de la file
#[tokio::test]
async fn list_actions_filters_by_state_in_queue_order() {
    within(async {
        let f = Fixture::start().await;
        let read = ActionKind::MarkRead { read: true };
        let first = f.apply(read, &["msg-1"], "k1").await;
        let flag = ActionKind::MarkFlagged { flagged: true };
        let second = f.apply(flag, &["msg-2"], "k2").await;
        f.run(cancel(&first[0])).await;
        let pending = f.actions(&[ActionState::Pending]).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, second[0]);
        let cancelled = f.actions(&[ActionState::Cancelled]).await;
        assert_eq!(cancelled[0].id, first[0]);
        let all = f.actions(&[]).await;
        assert_eq!(all.len(), 2);
        assert!(!f.message("msg-1").flags.seen);
    })
    .await;
}

/// Une action déjà appliquée ne s'annule plus ni ne se relance ; une action inconnue est introuvable
#[tokio::test]
async fn cancel_and_retry_are_refused_for_applied_or_unknown_actions() {
    within(async {
        let f = Fixture::start().await;
        let ids = f
            .apply(ActionKind::MarkRead { read: true }, &["msg-1"], "k1")
            .await;
        f.advance(5_001);
        f.wait_for_state(&ids[0], ActionState::Succeeded).await;
        let late = f.refused(cancel(&ids[0])).await;
        assert_eq!(late.code, "incompatible_state.action_not_cancellable");
        let retried = f.refused(retry(&ids[0])).await;
        assert_eq!(retried.code, "incompatible_state.action_not_retryable");
        let unknown = ActionId::from_raw("action-inconnue");
        assert_eq!(f.refused(cancel(&unknown)).await.code, "not_found.action");
    })
    .await;
}

/// Relancer une action échouée réapplique son effet local, puis l'échec définitif le défait de nouveau
#[tokio::test]
async fn retry_failed_action_reapplies_local_effect_then_reverts_again() {
    within(async {
        let mut f = Fixture::start().await;
        let ids = f.apply(move_to("box-arch"), &["msg-2"], "k1").await;
        f.store.expunge(support_write::USER, "INBOX", 2);
        f.advance(5_001);
        f.wait_for_state(&ids[0], ActionState::Failed).await;
        assert_eq!(f.boxes_of("msg-2"), ["box-in"]);
        f.go_offline().await;
        f.run(retry(&ids[0])).await;
        assert_eq!(f.state_of(&ids[0]), ActionState::Pending);
        assert_eq!(f.boxes_of("msg-2"), ["box-arch"]);
        f.go_online().await;
        f.wait_for_state(&ids[0], ActionState::Failed).await;
        assert_eq!(f.boxes_of("msg-2"), ["box-in"]);
    })
    .await;
}

/// Lister la file d'un compte inconnu est refusé
#[tokio::test]
async fn list_actions_of_an_unknown_account_is_refused() {
    within(async {
        let f = Fixture::start().await;
        let request = Request::Queue(QueueRequest::ListActions {
            account: AccountId::from_raw("compte-inconnu"),
            states: Vec::new(),
        });
        assert_eq!(f.refused(request).await.code, "not_found.account");
    })
    .await;
}
