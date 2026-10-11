#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Soumission idempotente (invariants 17 et 18) : la même clé n'a qu'un effet et qu'une action.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::submit;
use mira_vocabulary::{ActionKind, ActionState, EventKind, Flags, IdempotencyKey, MessageId};
use support_actions::inspect::{action, message, queue_len};
use support_actions::quirks::folders_quirks;
use support_actions::requests::{request, submit_folders};
use support_actions::world::World;

/// Invariant 18 : une seconde soumission sous la même clé rend la même action et n'applique rien de plus
#[test]
fn invariant_18_resubmission_with_same_key_has_single_effect() {
    let world = World::seeded();
    let kind = ActionKind::MarkRead { read: true };
    let first = submit_folders(&world, kind.clone(), &["msg-1"], "key-1");
    assert!(message(&world.db, "msg-1").flags.seen);
    world
        .db
        .write(|tx| tx.update_message_flags(&MessageId::from_raw("msg-1"), Flags::default()))
        .unwrap();
    let second = submit_folders(&world, kind, &["msg-1"], "key-1");
    assert_eq!(first, second);
    assert_eq!(queue_len(&world.db), 1);
    assert!(
        !message(&world.db, "msg-1").flags.seen,
        "aucun second effet"
    );
}

/// Une seule notification de mise en file, même si la commande est rejouée
#[test]
fn resubmission_emits_a_single_queued_event() {
    let world = World::seeded();
    let kind = ActionKind::MarkFlagged { flagged: true };
    let id = submit_folders(&world, kind.clone(), &["msg-2"], "key-flag");
    submit_folders(&world, kind, &["msg-2"], "key-flag");
    let kinds = world.sink.kinds();
    assert_eq!(kinds.len(), 1);
    assert_eq!(
        kinds[0],
        EventKind::ActionQueued {
            account: mira_vocabulary::AccountId::from_raw("acc-1"),
            action: id,
        }
    );
}

/// Une clé réutilisée pour une autre commande est refusée, sans effet ni action de plus
#[test]
fn reused_key_for_another_command_is_refused_without_effect() {
    let world = World::seeded();
    submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "key-x",
    );
    let other = request(
        ActionKind::MarkFlagged { flagged: true },
        &["msg-1"],
        "key-x",
    );
    let error = submit(&world.ctx(), &folders_quirks(), other).unwrap_err();
    assert_eq!(error.code, "invalid_input.idempotency_conflict");
    assert!(!message(&world.db, "msg-1").flags.flagged);
    assert_eq!(queue_len(&world.db), 1);
}

/// Deux clés différentes donnent deux actions distinctes, dans l'ordre de soumission
#[test]
fn distinct_keys_queue_distinct_actions_in_order() {
    let world = World::seeded();
    let first = submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "key-a",
    );
    let second = submit_folders(
        &world,
        ActionKind::MarkRead { read: false },
        &["msg-1"],
        "key-b",
    );
    assert_ne!(first, second);
    let queued = world
        .db
        .read(|tx| tx.list_actions(&mira_vocabulary::AccountId::from_raw("acc-1"), &[]))
        .unwrap();
    assert_eq!(queued[0].id, first);
    assert_eq!(queued[1].id, second);
    assert_eq!(action(&world.db, &first).state, ActionState::Pending);
    assert_eq!(
        action(&world.db, &second).idempotency_key,
        IdempotencyKey("key-b".into())
    );
}
