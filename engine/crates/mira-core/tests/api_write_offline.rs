#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Écritures par la façade hors ligne : retour immédiat, rejeu dans l'ordre, échec définitif (inv. 19, 22, 28, S7, S8).

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_write;

use mira_vocabulary::{AccountState, ActionKind, ActionState, EventKind, MailboxId};
use support_api::queries::event_kinds;
use support_imap::within;
use support_write::fixture::Fixture;
use support_write::mailbox_builders::move_to;
use support_write::waits::eventually;

/// Inv. 28 : serveur coupé, l'action rend son identifiant sans réseau et l'effet local est visible
#[tokio::test]
async fn invariant_28_actions_return_while_offline() {
    within(async {
        let f = Fixture::start().await;
        f.go_offline().await;
        let ids = f
            .apply(ActionKind::MarkRead { read: true }, &["msg-1"], "k1")
            .await;
        assert_eq!(ids.len(), 1);
        assert!(f.message("msg-1").flags.seen);
        assert_eq!(f.state_of(&ids[0]), ActionState::Pending);
        assert!(!f.server_seen("INBOX", 1));
        f.advance(5_001);
        eventually("compte hors ligne", || {
            f.account_state() == AccountState::Offline
        })
        .await;
        assert_eq!(f.state_of(&ids[0]), ActionState::Pending);
    })
    .await;
}

/// S7 : déplacer hors ligne puis rejouer au retour en ligne, dans l'ordre de soumission
#[tokio::test]
async fn scenario_s7_move_offline_then_replay_in_order() {
    within(async {
        let mut f = Fixture::start().await;
        f.go_offline().await;
        let first = f.apply(move_to("box-arch"), &["msg-1"], "k1").await;
        let read = ActionKind::MarkRead { read: true };
        let second = f.apply(read, &["msg-2"], "k2").await;
        let third = f.apply(move_to("box-trash"), &["msg-2"], "k3").await;
        assert_eq!(f.boxes_of("msg-1"), ["box-arch"]);
        let queued = f.actions(&[ActionState::Pending]).await;
        let order: Vec<_> = queued.into_iter().map(|action| action.id).collect();
        assert_eq!(order, [&first, &second, &third].map(|ids| ids[0].clone()));
        f.advance(5_001);
        f.wait_for_account(AccountState::Offline).await;
        f.go_online().await;
        f.wait_for_state(&third[0], ActionState::Succeeded).await;
        assert_eq!(f.server_uids("INBOX"), vec![3]);
        assert_eq!(f.server_uids("Archive").len(), 1);
        assert!(f.server_seen("Trash", f.server_uids("Trash")[0]));
        f.wait_for_account(AccountState::Connected).await;
    })
    .await;
}

/// S8 : message supprimé côté serveur avant le rejeu, l'action échoue et l'effet local est annulé
#[tokio::test]
async fn scenario_s8_action_on_deleted_message_fails_and_reverts() {
    within(async {
        let f = Fixture::start().await;
        let to_archive = ActionKind::Move {
            to: MailboxId::from_raw("box-arch"),
        };
        let ids = f.apply(to_archive, &["msg-2"], "k1").await;
        assert_eq!(
            f.message("msg-2").mailboxes,
            vec![MailboxId::from_raw("box-arch")]
        );
        f.store.expunge(support_write::USER, "INBOX", 2);
        f.advance(5_001);
        f.wait_for_state(&ids[0], ActionState::Failed).await;
        assert_eq!(
            f.message("msg-2").mailboxes,
            vec![MailboxId::from_raw("box-in")]
        );
        let failed = |kind: &EventKind| matches!(kind, EventKind::ActionFailed { .. });
        assert!(event_kinds(f.engine()).await.iter().any(failed));
    })
    .await;
}

/// Le lot est atomique : une cible inconnue annule tout, aucune action ni effet local
#[tokio::test]
async fn apply_action_batch_is_atomic_at_submission() {
    within(async {
        let f = Fixture::start().await;
        let request = support_write::builders::apply_request(
            ActionKind::MarkRead { read: true },
            &["msg-1", "msg-2", "msg-inconnu"],
            "k1",
        );
        let error = f.refused(request).await;
        assert_eq!(error.code, "not_found.message");
        assert!(f.actions(&[]).await.is_empty());
        assert!(!f.message("msg-1").flags.seen);
        let ids = f
            .apply(
                ActionKind::MarkRead { read: true },
                &["msg-1", "msg-2"],
                "k2",
            )
            .await;
        assert_eq!(ids.len(), 1);
        assert!(f.message("msg-1").flags.seen && f.message("msg-2").flags.seen);
    })
    .await;
}

/// Même clé, même action : aucun second effet ni second enregistrement (inv. 18)
#[tokio::test]
async fn apply_action_with_same_key_is_idempotent() {
    within(async {
        let f = Fixture::start().await;
        let first = f
            .apply(ActionKind::MarkFlagged { flagged: true }, &["msg-1"], "k1")
            .await;
        let again = f
            .apply(ActionKind::MarkFlagged { flagged: true }, &["msg-1"], "k1")
            .await;
        assert_eq!(first, again);
        assert_eq!(f.actions(&[]).await.len(), 1);
        let queued = |kind: &EventKind| matches!(kind, EventKind::ActionQueued { .. });
        assert_eq!(
            event_kinds(f.engine())
                .await
                .iter()
                .filter(|k| queued(k))
                .count(),
            1
        );
        f.advance(5_001);
        f.wait_for_state(&first[0], ActionState::Succeeded).await;
        assert!(
            f.server_flags("INBOX", 1)
                .iter()
                .any(|flag| flag.contains("Flagged"))
        );
    })
    .await;
}

/// UndoAction défait l'effet local dans le délai ; passé le délai, l'action n'est plus annulable
#[tokio::test]
async fn undo_action_restores_local_state_within_delay() {
    within(async {
        let f = Fixture::start().await;
        let ids = f
            .apply(ActionKind::MarkRead { read: true }, &["msg-1"], "k1")
            .await;
        let undo = support_write::builders::undo_request(&ids[0]);
        f.run(undo.clone()).await;
        assert!(!f.message("msg-1").flags.seen);
        assert_eq!(f.state_of(&ids[0]), ActionState::Cancelled);
        assert_eq!(
            f.refused(undo).await.code,
            "incompatible_state.action_not_cancellable"
        );
        let kept = f
            .apply(ActionKind::MarkRead { read: true }, &["msg-2"], "k2")
            .await;
        f.advance(5_001);
        f.wait_for_state(&kept[0], ActionState::Succeeded).await;
        let late = support_write::builders::undo_request(&kept[0]);
        assert_eq!(
            f.refused(late).await.code,
            "incompatible_state.action_not_cancellable"
        );
    })
    .await;
}
