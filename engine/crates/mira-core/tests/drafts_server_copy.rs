#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Mise en file de la copie serveur d'un brouillon.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::drafts::queue_draft_copy;
use mira_vocabulary::{ActionKind, DraftId, DraftState, IdempotencyKey, Timestamp};
use support_actions::drafts::blank;
use support_actions::world::World;

/// Un brouillon en édition donne une action `SaveDraft`, la même clé rend la même action
#[test]
fn queue_creates_one_save_action_per_key() {
    let world = World::seeded();
    let draft = blank(&world);
    let key = IdempotencyKey("copy-1".to_string());
    let first = queue_draft_copy(&world.ctx(), &draft.id, key.clone()).unwrap();
    let again = queue_draft_copy(&world.ctx(), &draft.id, key).unwrap();
    assert_eq!(first, again);
    let action = world.db.read(|tx| tx.get_action(&first)).unwrap().unwrap();
    assert!(matches!(action.kind, ActionKind::SaveDraft { draft: ref id } if *id == draft.id));
}

/// Un brouillon inconnu ou déjà planifié est refusé
#[test]
fn queue_refuses_unknown_and_scheduled_drafts() {
    let world = World::seeded();
    let key = IdempotencyKey("copy-2".to_string());
    let unknown = queue_draft_copy(&world.ctx(), &DraftId::from_raw("nope"), key.clone());
    assert_eq!(unknown.unwrap_err().code, "not_found.draft");
    let mut scheduled = blank(&world);
    scheduled.state = DraftState::Scheduled {
        at: Timestamp::from_millis(1),
    };
    world.db.write(|tx| tx.update_draft(&scheduled)).unwrap();
    let error = queue_draft_copy(&world.ctx(), &scheduled.id, key).unwrap_err();
    assert_eq!(error.code, "incompatible_state.draft_not_editable");
}
