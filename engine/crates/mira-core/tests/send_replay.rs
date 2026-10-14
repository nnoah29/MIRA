#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Envoi par le rejeu : S9 (annulation puis envoi), copie dans Envoyés, erreurs SMTP classées.

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{ReplayStop, SEND_UNDO_DELAY_MILLIS};
use mira_core::drafts::{cancel_send, schedule_send};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ActionState, DraftState, IdempotencyKey, ProviderQuirks};
use support_actions::drafts::stored;
use support_imap::within;
use support_replay::events_view::{send_events, send_failure_code};
use support_replay::fixture::Fixture;
use support_replay::observe::server_message_ids;
use support_replay::presets::standard as started;
use support_replay::queue_view::state_of;
use support_replay::sends::{
    assert_sent_once_and_draft_gone, schedule_ready_draft, stored_message_id,
};

/// S9 : planifier, annuler pendant le délai (rien ne part), replanifier, puis l'envoi part après 10 s
#[tokio::test]
async fn scenario_s9_send_with_undo_then_send() {
    within(async {
        let fx = started().await;
        let (draft, first) = schedule_ready_draft(&fx, "k-s9-a");
        fx.advance(5_000);
        cancel_send(&fx.world.ctx(), &draft.id).unwrap();
        assert_eq!(stored(&fx.world, &draft).state, DraftState::Editing);
        assert_eq!(state_of(&fx, &first), ActionState::Cancelled);
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        assert_eq!(fx.replay(&mut client).await.succeeded, 0);
        assert!(fx.smtp.received().is_empty());
        let key = IdempotencyKey("k-s9-b".to_string());
        let second = schedule_send(&fx.world.ctx(), &draft.id, None, key).unwrap();
        fx.advance(SEND_UNDO_DELAY_MILLIS - 1);
        assert_eq!(fx.replay(&mut client).await.succeeded, 0);
        fx.advance(1);
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        assert_sent_once_and_draft_gone(&fx, &second, &draft);
        let events = ["scheduled", "scheduled", "departed", "sent_copy"];
        assert_eq!(send_events(&fx), events);
    })
    .await;
}

/// Une copie est ajoutée dans Envoyés seulement si le serveur n'en garde pas lui-même
#[tokio::test]
async fn sent_copy_is_appended_only_when_the_server_does_not_keep_one() {
    within(async {
        let fx = started().await;
        let (_, action) = schedule_ready_draft(&fx, "k-copy");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        fx.replay(&mut client).await;
        let wanted = stored_message_id(&fx, &action).unwrap();
        let expected = [wanted.trim_matches(['<', '>'])];
        assert_eq!(server_message_ids(&fx, "Sent"), expected);
        let quirks = ProviderQuirks {
            server_saves_sent_copy: true,
            ..ProviderQuirks::default()
        };
        let keeper = Fixture::start(CapabilitySet::full(), quirks).await;
        schedule_ready_draft(&keeper, "k-copy-2");
        keeper.advance(SEND_UNDO_DELAY_MILLIS);
        let mut other = keeper.client().await;
        assert_eq!(keeper.replay(&mut other).await.succeeded, 1);
        assert!(server_message_ids(&keeper, "Sent").is_empty());
        assert_eq!(send_events(&keeper), ["scheduled", "departed"]);
    })
    .await;
}

/// 552 : message trop gros, échec définitif ; le brouillon revient en édition et rien n'est renvoyé
#[tokio::test]
async fn too_large_message_fails_for_good_and_returns_the_draft_to_editing() {
    within(async {
        let fx = started().await;
        let (draft, action) = schedule_ready_draft(&fx, "k-552");
        fx.smtp.fail_next("552 5.3.4 Message too big");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!((report.failed, report.succeeded), (1, 0));
        assert_eq!(state_of(&fx, &action), ActionState::Failed);
        assert_eq!(stored(&fx.world, &draft).state, DraftState::Editing);
        assert_eq!(send_events(&fx), ["scheduled", "failed"]);
        let code = send_failure_code(&fx);
        assert_eq!(code.as_deref(), Some("server.message_too_large"));
        fx.advance(3_600_000);
        assert_eq!(fx.replay(&mut client).await.succeeded, 0);
        assert!(fx.smtp.received().is_empty());
    })
    .await;
}

/// Autre 5xx : refusé par le serveur, échec définitif
#[tokio::test]
async fn permanent_smtp_rejection_is_final() {
    within(async {
        let fx = started().await;
        let (draft, action) = schedule_ready_draft(&fx, "k-550");
        fx.smtp.fail_next("550 5.7.1 Rejected by policy");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        assert_eq!(fx.replay(&mut client).await.failed, 1);
        assert_eq!(state_of(&fx, &action), ActionState::Failed);
        assert_eq!(stored(&fx.world, &draft).state, DraftState::Editing);
        assert_eq!(send_failure_code(&fx).as_deref(), Some("server.rejected"));
        assert!(server_message_ids(&fx, "Sent").is_empty());
    })
    .await;
}

/// 4xx : passagère ; l'envoi reste en attente avec un délai, le brouillon reste planifié, puis il part
#[tokio::test]
async fn transient_smtp_rejection_is_retried_later() {
    within(async {
        let fx = started().await;
        let (draft, action) = schedule_ready_draft(&fx, "k-451");
        fx.smtp.fail_next("451 4.7.1 Greylisted, try later");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert!(matches!(report.stopped, Some(ReplayStop::Retry { .. })));
        assert_eq!(state_of(&fx, &action), ActionState::Pending);
        let state = stored(&fx.world, &draft).state;
        assert!(matches!(state, DraftState::Scheduled { .. }));
        assert!(fx.smtp.received().is_empty());
        fx.advance(5_000);
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        assert_eq!(fx.smtp.received().len(), 1);
    })
    .await;
}
