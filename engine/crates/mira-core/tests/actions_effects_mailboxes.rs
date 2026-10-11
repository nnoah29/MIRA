#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Effets locaux sur les boîtes : créer, renommer (avec descendantes), supprimer, vider.

mod support;
mod support_actions;
mod support_mirror;

use mira_vocabulary::{AccountId, ActionKind, MailboxId, MailboxRole};
use support_actions::inspect::{boxes_of, mailbox};
use support_actions::requests::submit_folders;
use support_actions::world::World;

/// Noms des boîtes d'acc-1, triés
fn names(world: &World) -> Vec<String> {
    let boxes = world
        .db
        .read(|tx| tx.list_mailboxes(&AccountId::from_raw("acc-1")))
        .unwrap();
    boxes.into_iter().map(|b| b.name).collect()
}

/// Créer une boîte l'ajoute au miroir, vide, avec le rôle « autre »
#[test]
fn create_mailbox_adds_an_empty_local_mailbox() {
    let world = World::seeded();
    let kind = ActionKind::CreateMailbox {
        name: "Factures".to_string(),
        parent: None,
    };
    submit_folders(&world, kind, &[], "k1");
    let created = world
        .db
        .read(|tx| tx.find_mailbox_by_name(&AccountId::from_raw("acc-1"), "Factures"))
        .unwrap()
        .unwrap();
    assert_eq!(created.role, MailboxRole::Other);
    assert_eq!(created.total, 0);
    assert!(created.marker.is_none());
}

/// Créer sous un parent compose le nom avec le séparateur de hiérarchie du parent
#[test]
fn create_mailbox_under_a_parent_uses_the_hierarchy_delimiter() {
    let world = World::seeded();
    let kind = ActionKind::CreateMailbox {
        name: "2026".to_string(),
        parent: Some(MailboxId::from_raw("box-proj")),
    };
    submit_folders(&world, kind, &[], "k1");
    assert!(names(&world).contains(&"Projects/2026".to_string()));
    let child = world
        .db
        .read(|tx| tx.find_mailbox_by_name(&AccountId::from_raw("acc-1"), "Projects/2026"))
        .unwrap()
        .unwrap();
    assert_eq!(child.parent, Some(MailboxId::from_raw("box-proj")));
    assert_eq!(child.display_name, "2026");
}

/// Renommer change le dernier segment du nom et le préfixe des boîtes descendantes
#[test]
fn rename_mailbox_updates_the_name_and_its_descendants() {
    let world = World::seeded();
    submit_folders(
        &world,
        ActionKind::CreateMailbox {
            name: "2026".to_string(),
            parent: Some(MailboxId::from_raw("box-proj")),
        },
        &[],
        "k1",
    );
    let kind = ActionKind::RenameMailbox {
        mailbox: MailboxId::from_raw("box-proj"),
        name: "Chantiers".to_string(),
    };
    submit_folders(&world, kind, &[], "k2");
    let names = names(&world);
    assert!(names.contains(&"Chantiers".to_string()));
    assert!(names.contains(&"Chantiers/2026".to_string()));
    assert!(!names.contains(&"Projects".to_string()));
    assert_eq!(mailbox(&world.db, "box-proj").display_name, "Chantiers");
}

/// Supprimer une boîte retire la boîte et ses apparitions ; les messages restent dans leurs autres boîtes
#[test]
fn delete_mailbox_removes_it_and_its_appearances() {
    let world = World::seeded();
    let kind = ActionKind::DeleteMailbox {
        mailbox: MailboxId::from_raw("box-work"),
    };
    submit_folders(&world, kind, &[], "k1");
    assert!(!names(&world).contains(&"Work".to_string()));
    assert_eq!(boxes_of(&world.db, "msg-3"), ["box-in"]);
}

/// Vider une boîte marque toutes ses apparitions « supprimées » sans toucher aux autres boîtes
#[test]
fn empty_mailbox_hides_all_its_messages_only_there() {
    let world = World::seeded();
    let kind = ActionKind::EmptyMailbox {
        mailbox: MailboxId::from_raw("box-in"),
    };
    submit_folders(&world, kind, &[], "k1");
    assert_eq!(mailbox(&world.db, "box-in").total, 0);
    assert_eq!(mailbox(&world.db, "box-in").unread, 0);
    assert_eq!(boxes_of(&world.db, "msg-3"), ["box-work"]);
    assert!(boxes_of(&world.db, "msg-1").is_empty());
}
