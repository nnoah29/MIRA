#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rejeu d'un `schedule_send` avec la même clé après un envoi réussi : l'action existante, aucun second envoi.

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::SEND_UNDO_DELAY_MILLIS;
use mira_core::drafts::schedule_send;
use mira_vocabulary::{ActionState, IdempotencyKey};
use support_imap::within;
use support_replay::presets::standard as started;
use support_replay::queue_view::state_of;
use support_replay::sends::{received_texts, schedule_ready_draft};

/// La même clé après le succès rend l'action déjà enregistrée, sans erreur ni nouvel envoi
#[tokio::test]
async fn regression_schedule_send_with_same_key_after_success_returns_the_existing_action() {
    within(async {
        let fx = started().await;
        let (draft, action) = schedule_ready_draft(&fx, "k-twice");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        let key = IdempotencyKey("k-twice".to_string());
        let again = schedule_send(&fx.world.ctx(), &draft.id, None, key).unwrap();
        assert_eq!(again, action);
        assert_eq!(state_of(&fx, &action), ActionState::Succeeded);
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        assert_eq!(fx.replay(&mut client).await.succeeded, 0);
        assert_eq!(received_texts(&fx).len(), 1);
    })
    .await;
}

/// Un brouillon inconnu avec une clé inconnue reste refusé
#[tokio::test]
async fn schedule_send_of_unknown_draft_with_new_key_is_still_refused() {
    within(async {
        let fx = started().await;
        let unknown = mira_vocabulary::DraftId::from_raw("absent");
        let key = IdempotencyKey("k-new".to_string());
        let error = schedule_send(&fx.world.ctx(), &unknown, None, key).unwrap_err();
        assert_eq!(error.code, "not_found.draft");
    })
    .await;
}
