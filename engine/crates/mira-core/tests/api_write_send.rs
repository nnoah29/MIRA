#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Envoi par la façade : annulation pendant le délai, envoi planifié, refus du serveur, copie dans Envoyés (S9, 10.8).

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_write;

use mira_core::api::{ComposeRequest, QueueRequest, Request};
use mira_core::ports::Clock;
use mira_vocabulary::{ActionState, DraftState, EventKind};
use support_api::queries::event_kinds;
use support_imap::within;
use support_write::builders::{cancel_send_request, key, send_request};
use support_write::fixture::Fixture;
use support_write::timing::settle;

/// S9 : envoi annulé pendant son délai (brouillon en édition, rien d'envoyé), puis renvoyé et parti
#[tokio::test]
async fn scenario_s9_send_with_undo_then_send() {
    within(async {
        let f = Fixture::start().await;
        let draft = f.ready_draft().await;
        let first = f.send(&draft.id, "k1").await;
        let state = f.get_draft(&draft.id).await.state;
        assert!(matches!(state, DraftState::Scheduled { .. }));
        let outbox = f.outbox().await;
        assert_eq!((outbox.len(), outbox[0].state), (1, ActionState::Pending));
        assert_eq!(outbox[0].subject, "Bonjour Bob");
        f.run(cancel_send_request(&first)).await;
        assert_eq!(f.get_draft(&draft.id).await.state, DraftState::Editing);
        assert!(f.outbox().await.is_empty());
        f.advance(10_001);
        settle().await;
        assert!(f.smtp.received().is_empty());
        let second = f.send(&draft.id, "k2").await;
        f.advance(10_001);
        f.wait_for_state(&second, ActionState::Succeeded).await;
        assert_eq!(f.smtp.received().len(), 1);
        assert_eq!(f.smtp.received()[0].recipients, ["bob@example.org"]);
        assert_eq!(f.server_uids("Sent").len(), 1);
    })
    .await;
}

/// Après l'envoi : brouillon supprimé, boîte d'envoi vide, événements d'envoi dans l'ordre
#[tokio::test]
async fn sent_draft_disappears_and_events_are_emitted() {
    within(async {
        let f = Fixture::start().await;
        let draft = f.ready_draft().await;
        let action = f.send(&draft.id, "k1").await;
        f.advance(10_001);
        f.wait_for_state(&action, ActionState::Succeeded).await;
        let missing = ComposeRequest::GetDraft {
            draft: draft.id.clone(),
        };
        assert_eq!(
            f.refused(Request::Compose(missing)).await.code,
            "not_found.draft"
        );
        assert!(f.outbox().await.is_empty());
        let kinds = event_kinds(f.engine()).await;
        let departed = kinds
            .iter()
            .position(|k| matches!(k, EventKind::SendDeparted { .. }));
        let scheduled = kinds
            .iter()
            .position(|k| matches!(k, EventKind::SendScheduled { .. }));
        assert!(scheduled.unwrap() < departed.unwrap());
    })
    .await;
}

/// La même clé rend la même action : un seul envoi, un seul message chez le serveur SMTP
#[tokio::test]
async fn send_draft_with_same_key_is_idempotent() {
    within(async {
        let f = Fixture::start().await;
        let draft = f.ready_draft().await;
        let first = f.send(&draft.id, "k1").await;
        assert_eq!(f.send(&draft.id, "k1").await, first);
        f.advance(10_001);
        f.wait_for_state(&first, ActionState::Succeeded).await;
        settle().await;
        assert_eq!(f.smtp.received().len(), 1);
        assert_eq!(f.actions(&[]).await.len(), 1);
    })
    .await;
}

/// Envoi planifié : rien ne part avant la date choisie
#[tokio::test]
async fn scheduled_send_waits_for_its_date() {
    within(async {
        let f = Fixture::start().await;
        let draft = f.ready_draft().await;
        let later = f.clock.now().plus_millis(3_600_000);
        let request = ComposeRequest::SendDraft {
            draft: draft.id.clone(),
            at: Some(later),
            idempotency_key: key("k1"),
        };
        let action = match f.run(Request::Compose(request)).await {
            mira_core::api::Response::Write(mira_core::api::WriteResponse::Action(id)) => id,
            other => panic!("réponse inattendue : {other:?}"),
        };
        f.advance(1_800_000);
        settle().await;
        assert!(f.smtp.received().is_empty());
        f.advance(1_800_001);
        f.wait_for_state(&action, ActionState::Succeeded).await;
        assert_eq!(f.smtp.received().len(), 1);
    })
    .await;
}

/// Refus définitif du serveur : brouillon en édition, envoi échoué dans la boîte d'envoi, relance possible
#[tokio::test]
async fn rejected_send_fails_then_retry_sends() {
    within(async {
        let f = Fixture::start().await;
        let draft = f.ready_draft().await;
        let action = f.send(&draft.id, "k1").await;
        f.smtp.fail_next("550 5.7.1 Rejected by policy");
        f.advance(10_001);
        f.wait_for_state(&action, ActionState::Failed).await;
        assert_eq!(f.get_draft(&draft.id).await.state, DraftState::Editing);
        assert_eq!(f.outbox().await[0].state, ActionState::Failed);
        let retry = Request::Queue(QueueRequest::RetryAction {
            action: action.clone(),
        });
        f.run(retry).await;
        f.wait_for_state(&action, ActionState::Succeeded).await;
        assert_eq!(f.smtp.received().len(), 1);
    })
    .await;
}

/// Un brouillon sans destinataire, ou déjà planifié, ne peut pas être envoyé
#[tokio::test]
async fn send_is_refused_without_recipient_or_when_already_scheduled() {
    within(async {
        let f = Fixture::start().await;
        let blank = f
            .draft_of(ComposeRequest::CreateDraft {
                account: f.account.clone(),
                identity: None,
                mode: mira_core::api::DraftMode::New,
            })
            .await;
        let error = f.refused(send_request(&blank.id, "k1")).await;
        assert_eq!(error.code, "invalid_input.missing_parameter");
        let ready = f.ready_draft().await;
        f.send(&ready.id, "k2").await;
        let again = f.refused(send_request(&ready.id, "k3")).await;
        assert_eq!(again.code, "incompatible_state.draft_not_editable");
    })
    .await;
}
