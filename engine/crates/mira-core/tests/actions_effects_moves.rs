#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Effets locaux des déplacements : déplacer, corbeille, spam, suppression définitive.

mod support;
mod support_actions;
mod support_mirror;

use mira_vocabulary::{ActionKind, MailboxId, MessageId};
use support_actions::inspect::{boxes_of, mailbox};
use support_actions::requests::submit_folders;
use support_actions::world::World;

/// Déplacer retire le message de ses boîtes visibles, l'ajoute à la cible (UID provisoire) et met les compteurs à jour
#[test]
fn move_relocates_the_message_and_updates_both_counters() {
    let world = World::seeded();
    let to = MailboxId::from_raw("box-proj");
    submit_folders(&world, ActionKind::Move { to }, &["msg-3"], "k1");
    assert_eq!(boxes_of(&world.db, "msg-3"), ["box-proj"]);
    assert_eq!(mailbox(&world.db, "box-in").total, 2);
    assert_eq!(mailbox(&world.db, "box-work").total, 0);
    assert_eq!(mailbox(&world.db, "box-proj").total, 1);
    assert_eq!(mailbox(&world.db, "box-proj").unread, 1);
    let appearance = world
        .db
        .read(|tx| {
            tx.get_appearance(
                &MessageId::from_raw("msg-3"),
                &MailboxId::from_raw("box-proj"),
            )
        })
        .unwrap()
        .unwrap();
    assert_eq!(appearance.uid, 0, "apparition provisoire, sans UID serveur");
}

/// La corbeille est la boîte de rôle « corbeille » du compte
#[test]
fn trash_moves_the_message_to_the_trash_mailbox() {
    let world = World::seeded();
    submit_folders(&world, ActionKind::Trash, &["msg-1", "msg-2"], "k1");
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-trash"]);
    assert_eq!(boxes_of(&world.db, "msg-2"), ["box-trash"]);
    assert_eq!(mailbox(&world.db, "box-trash").total, 2);
    assert_eq!(mailbox(&world.db, "box-in").total, 1);
}

/// « Spam » déplace vers le courrier indésirable, « non spam » vers la réception
#[test]
fn spam_and_not_spam_move_between_junk_and_inbox() {
    let world = World::seeded();
    submit_folders(
        &world,
        ActionKind::MarkSpam { spam: true },
        &["msg-1"],
        "k1",
    );
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-junk"]);
    submit_folders(
        &world,
        ActionKind::MarkSpam { spam: false },
        &["msg-1"],
        "k2",
    );
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-in"]);
}

/// Sans boîte de corbeille connue, l'action est refusée avant la mise en file (`server.mailbox_missing`)
#[test]
fn trash_without_a_trash_mailbox_is_refused_before_queueing() {
    let world = World::seeded();
    world
        .db
        .write(|tx| tx.delete_mailbox(&MailboxId::from_raw("box-trash")))
        .unwrap();
    let request = support_actions::requests::request(ActionKind::Trash, &["msg-1"], "k1");
    let error = mira_core::actions::submit(
        &world.ctx(),
        &support_actions::quirks::folders_quirks(),
        request,
    )
    .unwrap_err();
    assert_eq!(error.code, "server.mailbox_missing");
    assert_eq!(support_actions::inspect::queue_len(&world.db), 0);
    assert_eq!(boxes_of(&world.db, "msg-1"), ["box-in"]);
}

/// Supprimer définitivement marque toutes les apparitions « supprimées » : le message disparaît des listes et compteurs
#[test]
fn delete_permanently_hides_the_message_everywhere() {
    let world = World::seeded();
    submit_folders(&world, ActionKind::DeletePermanently, &["msg-3"], "k1");
    assert!(boxes_of(&world.db, "msg-3").is_empty());
    assert_eq!(mailbox(&world.db, "box-in").total, 2);
    assert_eq!(mailbox(&world.db, "box-work").total, 0);
    let kept = world
        .db
        .read(|tx| tx.appearances_of_message(&MessageId::from_raw("msg-3")))
        .unwrap();
    assert!(kept.iter().all(|a| a.deleted && a.uid != 0));
}
