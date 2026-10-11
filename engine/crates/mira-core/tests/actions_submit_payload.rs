#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Soumission : cibles serveur et inverse mémorisés avec l'action (boîte, UIDVALIDITY, UID, identifiant stable).

mod support;
mod support_actions;
mod support_mirror;

use mira_core::actions::{ServerTargets, UndoRecord, load_server_targets};
use mira_vocabulary::{ActionId, ActionKind, MailboxId};
use support_actions::inspect::queue_len;
use support_actions::requests::submit_folders;
use support_actions::world::World;

/// Cibles serveur d'une action, relues en base
fn targets_of(world: &World, id: &ActionId) -> ServerTargets {
    world.db.read(|tx| load_server_targets(tx, id)).unwrap()
}

/// Enregistrement d'annulation d'une action, relu en JSON
fn undo_of(world: &World, id: &ActionId) -> UndoRecord {
    let payload = world.db.read(|tx| tx.get_action_payload(id)).unwrap();
    serde_json::from_str(&payload.unwrap().undo_json.unwrap()).unwrap()
}

/// Les cibles mémorisent l'identifiant stable et chaque apparition avec sa boîte, sa validité et son UID
#[test]
fn server_targets_record_stable_id_and_every_appearance() {
    let world = World::seeded();
    let id = submit_folders(
        &world,
        ActionKind::MarkRead { read: true },
        &["msg-1", "msg-3"],
        "key-t",
    );
    let targets = targets_of(&world, &id);
    assert_eq!(targets.messages.len(), 2);
    let first = &targets.messages[0];
    assert_eq!(first.stable_id.as_deref(), Some("gm-1"));
    assert_eq!(first.appearances.len(), 1);
    assert_eq!(first.appearances[0].mailbox.as_str(), "box-in");
    assert_eq!(first.appearances[0].mailbox_name, "INBOX");
    assert_eq!(first.appearances[0].uid_validity, Some(77));
    assert_eq!(first.appearances[0].uid, 1);
    let third = &targets.messages[1];
    assert_eq!(third.stable_id, None);
    assert_eq!(third.appearances.len(), 2);
}

/// Les cibles sont relevées avant l'effet : un déplacement garde la boîte d'origine
#[test]
fn server_targets_are_captured_before_the_local_effect() {
    let world = World::seeded();
    let to = MailboxId::from_raw("box-proj");
    let id = submit_folders(&world, ActionKind::Move { to }, &["msg-1"], "key-m");
    let targets = targets_of(&world, &id);
    assert_eq!(
        targets.messages[0].appearances[0].mailbox.as_str(),
        "box-in"
    );
    assert_eq!(targets.mailboxes.len(), 1);
    assert_eq!(targets.mailboxes[0].mailbox.as_str(), "box-proj");
    assert_eq!(targets.mailboxes[0].name, "Projects");
}

/// Une action sur une boîte relève son nom d'avant l'effet ; l'inverse est mémorisé avec l'intention
#[test]
fn mailbox_action_records_name_before_rename_and_an_inverse() {
    let world = World::seeded();
    let kind = ActionKind::RenameMailbox {
        mailbox: MailboxId::from_raw("box-proj"),
        name: "Chantiers".to_string(),
    };
    let id = submit_folders(&world, kind, &[], "key-r");
    let targets = targets_of(&world, &id);
    assert_eq!(targets.mailboxes[0].name, "Projects");
    let undo = undo_of(&world, &id);
    assert_eq!(undo.inverse.len(), 1);
    assert_eq!(queue_len(&world.db), 1);
}
