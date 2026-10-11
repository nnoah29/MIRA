#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Envoi planifié d'un brouillon : mise en file, idempotence, annulation, échec.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::{SEND_UNDO_DELAY_MILLIS, retry, revert_local_effect};
use mira_core::drafts::{DraftEdit, cancel_send, edit_draft, schedule_send};
use mira_vocabulary::{
    AccountId, ActionKind, ActionState, DraftState, EventKind, IdempotencyKey, Timestamp,
};
use support_actions::drafts::{blank, ready_draft, stored};
use support_actions::inspect::{action, force_state, queue_len};
use support_actions::world::World;

/// Sans date, l'envoi part dans 10 s : action `Send` en attente, brouillon planifié, événement émis
#[test]
fn schedule_send_defaults_to_a_ten_second_undo_delay() {
    let world = World::seeded();
    let draft = ready_draft(&world);
    let id = schedule_send(&world.ctx(), &draft.id, None, IdempotencyKey("s1".into())).unwrap();
    let queued = action(&world.db, &id);
    let at = Timestamp::from_millis(world.now().as_millis() + SEND_UNDO_DELAY_MILLIS);
    assert_eq!(
        queued.kind,
        ActionKind::Send {
            draft: draft.id.clone()
        }
    );
    assert_eq!(
        (queued.state, queued.not_before),
        (ActionState::Pending, at)
    );
    assert_eq!(stored(&world, &draft).state, DraftState::Scheduled { at });
    let last = world.sink.kinds().pop().unwrap();
    assert_eq!(
        last,
        EventKind::SendScheduled {
            account: AccountId::from_raw("acc-1"),
            action: id,
            at
        }
    );
}

/// Avec une date, l'envoi est différé à cette date ; une date passée est ramenée à maintenant
#[test]
fn schedule_send_uses_the_requested_date() {
    let world = World::seeded();
    let draft = ready_draft(&world);
    let later = Timestamp::from_millis(world.now().as_millis() + 3_600_000);
    let id = schedule_send(
        &world.ctx(),
        &draft.id,
        Some(later),
        IdempotencyKey("s1".into()),
    )
    .unwrap();
    assert_eq!(action(&world.db, &id).not_before, later);
    let other = ready_draft(&world);
    let past = Timestamp::from_millis(1);
    let id = schedule_send(
        &world.ctx(),
        &other.id,
        Some(past),
        IdempotencyKey("s2".into()),
    )
    .unwrap();
    assert_eq!(action(&world.db, &id).not_before, world.now());
}

/// La même clé rend la même action sans second événement ; un brouillon sans destinataire est refusé
#[test]
fn schedule_send_is_idempotent_and_needs_a_recipient() {
    let world = World::seeded();
    let draft = ready_draft(&world);
    let key = IdempotencyKey("s1".into());
    let first = schedule_send(&world.ctx(), &draft.id, None, key.clone()).unwrap();
    let second = schedule_send(&world.ctx(), &draft.id, None, key).unwrap();
    assert_eq!(first, second);
    assert_eq!(queue_len(&world.db), 1);
    assert_eq!(world.sink.kinds().len(), 1);
    let again = schedule_send(&world.ctx(), &draft.id, None, IdempotencyKey("s2".into()));
    assert_eq!(
        again.unwrap_err().code,
        "incompatible_state.draft_not_editable"
    );
    let empty = blank(&world);
    let error = schedule_send(&world.ctx(), &empty.id, None, IdempotencyKey("s3".into()));
    assert_eq!(error.unwrap_err().code, "invalid_input.missing_parameter");
    assert_eq!(stored(&world, &empty).state, DraftState::Editing);
}

/// Annuler l'envoi pendant le délai remet le brouillon en édition et annule l'action
#[test]
fn cancel_send_returns_the_draft_to_editing() {
    let world = World::seeded();
    let draft = ready_draft(&world);
    let id = schedule_send(&world.ctx(), &draft.id, None, IdempotencyKey("s1".into())).unwrap();
    world.clock.advance_millis(4_000);
    cancel_send(&world.ctx(), &draft.id).unwrap();
    assert_eq!(stored(&world, &draft).state, DraftState::Editing);
    assert_eq!(action(&world.db, &id).state, ActionState::Cancelled);
    assert!(matches!(
        world.sink.kinds().pop(),
        Some(EventKind::ActionCancelled { .. })
    ));
    let edit = DraftEdit {
        subject: Some("Corrigé".into()),
        ..DraftEdit::default()
    };
    edit_draft(&world.ctx(), &draft.id, edit).unwrap();
    let resend = schedule_send(&world.ctx(), &draft.id, None, IdempotencyKey("s2".into()));
    assert!(resend.is_ok());
}

/// Un envoi déjà parti (en cours) ou absent ne s'annule pas
#[test]
fn cancel_send_refuses_when_nothing_is_pending() {
    let world = World::seeded();
    let draft = ready_draft(&world);
    let none = cancel_send(&world.ctx(), &draft.id).unwrap_err();
    assert_eq!(none.code, "incompatible_state.action_not_cancellable");
    let id = schedule_send(&world.ctx(), &draft.id, None, IdempotencyKey("s1".into())).unwrap();
    force_state(&world, &id, ActionState::InProgress);
    let started = cancel_send(&world.ctx(), &draft.id).unwrap_err();
    assert_eq!(started.code, "incompatible_state.action_not_cancellable");
    assert!(matches!(
        stored(&world, &draft).state,
        DraftState::Scheduled { .. }
    ));
}

/// Un échec définitif remet le brouillon en édition ; la relance le replanifie
#[test]
fn failed_send_reverts_to_editing_and_retry_reschedules() {
    let world = World::seeded();
    let draft = ready_draft(&world);
    let id = schedule_send(&world.ctx(), &draft.id, None, IdempotencyKey("s1".into())).unwrap();
    world
        .db
        .write(|tx| {
            let pending = tx.get_action(&id)?.unwrap();
            revert_local_effect(tx, &pending)?;
            tx.set_action_state(&id, ActionState::Failed)
        })
        .unwrap();
    assert_eq!(stored(&world, &draft).state, DraftState::Editing);
    retry(&world.ctx(), &id).unwrap();
    assert_eq!(action(&world.db, &id).state, ActionState::Pending);
    assert_eq!(
        stored(&world, &draft).state,
        DraftState::Scheduled { at: world.now() }
    );
}
