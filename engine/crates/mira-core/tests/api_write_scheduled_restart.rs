#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Envoi programmé et moteur fermé : l'envoi ne part pas pendant la fermeture, mais part une seule fois à la réouverture.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_write;

use mira_core::api::{ComposeRequest, Request, Response, WriteResponse};
use mira_core::ports::Clock;
use mira_vocabulary::{ActionId, ActionState, DraftId, DraftState, IdempotencyKey};
use support_imap::within;
use support_write::fixture::Fixture;
use support_write::timing::settle;

const ONE_HOUR: i64 = 3_600_000;

/// Programme l'envoi du brouillon dans une heure et rend l'action
async fn schedule_in_one_hour(f: &Fixture, draft: &DraftId) -> ActionId {
    let at = f.clock.now().plus_millis(ONE_HOUR);
    let request = Request::Compose(ComposeRequest::SendDraft {
        draft: draft.clone(),
        at: Some(at),
        idempotency_key: IdempotencyKey("k-later".to_string()),
    });
    match f.run(request).await {
        Response::Write(WriteResponse::Action(action)) => action,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Moteur rouvert avant l'heure : rien ne part, puis l'envoi part à l'heure prévue, une seule fois
#[tokio::test]
async fn scheduled_send_survives_a_restart_and_leaves_at_its_time() {
    within(async {
        let mut f = Fixture::start().await;
        let draft = f.ready_draft().await;
        let action = schedule_in_one_hour(&f, &draft.id).await;
        f.restart();
        settle().await;
        assert!(f.smtp.received().is_empty());
        assert!(matches!(
            f.get_draft(&draft.id).await.state,
            DraftState::Scheduled { .. }
        ));
        f.advance(ONE_HOUR + 1);
        f.wait_for_state(&action, ActionState::Succeeded).await;
        assert_eq!(f.smtp.received().len(), 1);
    })
    .await;
}

/// Moteur fermé pendant l'heure prévue : rien ne part tant qu'il est fermé, l'envoi part à la réouverture
#[tokio::test]
async fn scheduled_send_overdue_while_closed_leaves_when_reopened() {
    within(async {
        let mut f = Fixture::start().await;
        let draft = f.ready_draft().await;
        let action = schedule_in_one_hour(&f, &draft.id).await;
        if let Some(old) = f.engine_slot.take() {
            old.shutdown().unwrap();
        }
        f.advance(2 * ONE_HOUR);
        settle().await;
        assert!(f.smtp.received().is_empty());
        f.restart();
        f.wait_for_state(&action, ActionState::Succeeded).await;
        assert_eq!(f.smtp.received().len(), 1);
        settle().await;
        assert_eq!(f.smtp.received().len(), 1);
    })
    .await;
}
