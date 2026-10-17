#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Échecs du rejeu : message disparu (S8), refus du serveur (inv. 22), coupure réseau, jeton révoqué (S10).

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{ReplayStop, UNDO_DELAY_MILLIS};
use mira_vocabulary::{AccountState, ActionKind, ActionState, MailboxId};
use support_actions::inspect::{action, boxes_of, message};
use support_actions::requests::submit_folders;
use support_imap::within;
use support_replay::credentials::FakeCredentials;
use support_replay::events_view::failure_codes;
use support_replay::observe::command_count;
use support_replay::presets::standard as started;
use support_replay::queue_view::state_of;

/// S8 : le message a été supprimé ailleurs ; l'action échoue, l'effet local est annulé, l'utilisateur est prévenu
#[tokio::test]
async fn scenario_s8_action_on_deleted_message_fails_and_reverts() {
    within(async {
        let fx = started().await;
        let id = submit_folders(
            &fx.world,
            ActionKind::MarkRead { read: true },
            &["msg-1"],
            "k-s8",
        );
        assert!(message(&fx.world.db, "msg-1").flags.seen);
        fx.env.store.expunge("alice", "INBOX", 1);
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!((report.succeeded, report.failed), (0, 1));
        assert_eq!(state_of(&fx, &id), ActionState::Failed);
        assert!(!message(&fx.world.db, "msg-1").flags.seen);
        assert_eq!(failure_codes(&fx), ["not_found.message"]);
    })
    .await;
}

/// Inv. 22 : un refus définitif du serveur (quota) annule l'effet local d'un déplacement et informe
#[tokio::test]
async fn invariant_22_server_refusal_reverts_local_effect_and_informs() {
    within(async {
        let fx = started().await;
        let to = MailboxId::from_raw("box-proj");
        let id = submit_folders(&fx.world, ActionKind::Move { to }, &["msg-1"], "k-22");
        assert_eq!(boxes_of(&fx.world.db, "msg-1"), vec!["box-proj"]);
        fx.env
            .store
            .fail_next("UID MOVE", "NO [OVERQUOTA] mailbox full");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!(report.failed, 1);
        assert_eq!(state_of(&fx, &id), ActionState::Failed);
        assert_eq!(boxes_of(&fx.world.db, "msg-1"), vec!["box-in"]);
        assert_eq!(failure_codes(&fx), ["server.quota_exceeded"]);
        assert!(action(&fx.world.db, &id).last_error.is_some());
    })
    .await;
}

/// Une coupure réseau est passagère : l'action reste en attente avec un délai, l'effet local reste visible, puis elle aboutit
#[tokio::test]
async fn network_failure_keeps_the_action_pending_with_backoff_then_succeeds() {
    within(async {
        let fx = started().await;
        let read = ActionKind::MarkRead { read: true };
        let id = submit_folders(&fx.world, read, &["msg-1"], "k-net");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        fx.env.server.drop_all_connections();
        let report = fx.replay(&mut client).await;
        assert!(matches!(report.stopped, Some(ReplayStop::Retry { .. })));
        let waiting = action(&fx.world.db, &id);
        assert_eq!((waiting.state, waiting.attempts), (ActionState::Pending, 1));
        assert_eq!(waiting.not_before, fx.world.now().plus_millis(5_000));
        assert!(message(&fx.world.db, "msg-1").flags.seen);
        let mut early = fx.client().await;
        assert_eq!(fx.replay(&mut early).await.succeeded, 0);
        fx.advance(5_000);
        assert_eq!(fx.replay(&mut early).await.succeeded, 1);
        assert_eq!(state_of(&fx, &id), ActionState::Succeeded);
    })
    .await;
}

/// S10 : jeton expiré et révoqué ; le compte demande une authentification, les actions restent en file sans toucher au serveur
#[tokio::test]
async fn scenario_s10_token_expired_keeps_actions_queued() {
    within(async {
        let mut fx = started().await;
        fx.credentials = FakeCredentials::expired_and_revoked();
        let read = ActionKind::MarkRead { read: true };
        let first = submit_folders(&fx.world, read, &["msg-1"], "k-s10-a");
        let to = MailboxId::from_raw("box-proj");
        let second = submit_folders(&fx.world, ActionKind::Move { to }, &["msg-2"], "k-s10-b");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!(report.stopped, Some(ReplayStop::AuthRequired));
        assert_eq!(fx.credentials.renewals(), 1);
        assert_eq!(command_count(&fx, "STORE"), 0);
        assert_eq!(state_of(&fx, &first), ActionState::Pending);
        assert_eq!(state_of(&fx, &second), ActionState::Pending);
        let account = fx.world.db.read(|tx| tx.get_account(&fx.account)).unwrap();
        assert_eq!(account.unwrap().state, AccountState::AuthRequired);
        fx.credentials = FakeCredentials::valid();
        assert_eq!(fx.replay(&mut client).await.succeeded, 2);
    })
    .await;
}

/// Inv. 28 : compte hors ligne, soumettre et planifier un envoi rendent la main sans toucher à aucun serveur
#[tokio::test]
async fn invariant_28_write_returns_without_network() {
    within(async {
        let fx = started().await;
        fx.world
            .db
            .write(|tx| tx.set_account_state(&fx.account, &AccountState::Offline))
            .unwrap();
        let archive = submit_folders(&fx.world, ActionKind::Archive, &["msg-1"], "k-28");
        let (_, send) = support_replay::sends::schedule_ready_draft(&fx, "k-28-send");
        assert_eq!(state_of(&fx, &archive), ActionState::Pending);
        assert_eq!(state_of(&fx, &send), ActionState::Pending);
        assert_eq!(fx.env.server.connection_count(), 0);
        assert!(fx.smtp.received().is_empty());
    })
    .await;
}
