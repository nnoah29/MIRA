#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rebase (D-23) : le serveur gagne sur le miroir, les actions en attente sont rejouées par-dessus.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::revert_local_effect;
use mira_vocabulary::{ActionKind, ActionState, Flags, MailboxId, MessageId};
use support_actions::inspect::{boxes_of, force_state, mailbox, message};
use support_actions::requests::submit_folders;
use support_actions::server::{rebase, server_moves_msg_1_to_work, server_overwrites_flags};
use support_actions::world::World;

/// Après une synchro qui écrase les drapeaux, l'action « marquer lu » en attente est rejouée par-dessus
#[test]
fn rebase_after_flags_overwrite_reapplies_pending_read() {
    let world = World::seeded();
    submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "k1",
    );
    server_overwrites_flags(&world, "msg-1", Flags::default());
    world
        .db
        .write(|tx| tx.refresh_mailbox_counters(&MailboxId::from_raw("box-in")))
        .unwrap();
    assert!(!message(&world.db, "msg-1").flags.seen);
    assert_eq!(mailbox(&world.db, "box-in").unread, 3);
    rebase(&world);
    assert!(message(&world.db, "msg-1").flags.seen);
    assert_eq!(mailbox(&world.db, "box-in").unread, 2);
}

/// Seules les actions en attente ou en cours sont rejouées ; les autres états sont définitifs
#[test]
fn rebase_replays_pending_and_in_progress_only() {
    let world = World::seeded();
    let states = [
        ("msg-1", ActionState::Pending),
        ("msg-2", ActionState::InProgress),
        ("msg-3", ActionState::Succeeded),
    ];
    for (index, (message_id, state)) in states.into_iter().enumerate() {
        let key = format!("k{index}");
        let kind = ActionKind::MarkRead { read: true };
        let id = submit_folders(&world, kind, &[message_id], &key);
        force_state(&world, &id, state);
        server_overwrites_flags(&world, message_id, Flags::default());
    }
    rebase(&world);
    assert!(message(&world.db, "msg-1").flags.seen);
    assert!(message(&world.db, "msg-2").flags.seen);
    assert!(
        !message(&world.db, "msg-3").flags.seen,
        "réussie : le serveur gagne"
    );
}

/// Un message déplacé côté serveur : l'action locale est rejouée sur la nouvelle situation
#[test]
fn rebase_replays_a_move_on_the_new_server_situation() {
    let world = World::seeded();
    let to = MailboxId::from_raw("box-proj");
    submit_folders(&world, ActionKind::Move { to }, &["msg-1"], "k1");
    server_moves_msg_1_to_work(&world);
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-work"]);
    rebase(&world);
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-proj"]);
    assert_eq!(mailbox(&world.db, "box-work").total, 1);
    assert_eq!(mailbox(&world.db, "box-proj").total, 1);
}

/// Défaire après un rebase rend la situation du serveur, pas celle d'avant la soumission
#[test]
fn reverting_after_rebase_restores_the_server_situation() {
    let world = World::seeded();
    let to = MailboxId::from_raw("box-proj");
    let id = submit_folders(&world, ActionKind::Move { to }, &["msg-1"], "k1");
    server_moves_msg_1_to_work(&world);
    rebase(&world);
    world
        .db
        .write(|tx| {
            let action = tx.get_action(&id)?.unwrap();
            revert_local_effect(tx, &action)
        })
        .unwrap();
    let boxes = boxes_of(&world.db, "msg-1");
    assert!(boxes.contains(&"box-work".to_string()));
    assert!(!boxes.contains(&"box-proj".to_string()));
    let work = world
        .db
        .read(|tx| {
            tx.get_appearance(
                &MessageId::from_raw("msg-1"),
                &MailboxId::from_raw("box-work"),
            )
        })
        .unwrap()
        .unwrap();
    assert_eq!(work.uid, 9);
}

/// Rejouer le rebase deux fois ne change rien : l'inverse mémorisé ne grossit pas
#[test]
fn rebase_is_idempotent() {
    let world = World::seeded();
    let id = submit_folders(&world, ActionKind::Trash, &["msg-1", "msg-2"], "k1");
    let before = world.db.read(|tx| tx.get_action_payload(&id)).unwrap();
    rebase(&world);
    rebase(&world);
    let after = world.db.read(|tx| tx.get_action_payload(&id)).unwrap();
    assert_eq!(before, after);
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-trash"]);
    assert_eq!(mailbox(&world.db, "box-trash").total, 2);
}
