#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Table de correspondance des identifiants serveur (D-13) et recoupement par empreinte.

mod support;
mod support_mirror;

use mira_core::storage::repositories::server_ids::ServerIdKind;
use mira_vocabulary::{AccountId, MailboxId, MessageId, Timestamp};
use support_mirror::fixtures::sample_message;
use support_mirror::seeded::database_with_messages;

/// Un identifiant serveur retrouve son message ; un autre compte ou une autre nature ne le voit pas
#[test]
fn mapped_server_id_finds_message_within_account_and_kind() {
    let db = database_with_messages(2);
    let acc = AccountId::from_raw("acc-1");
    let first = MessageId::from_raw("msg-1");
    db.write(|tx| tx.map_server_id(&acc, ServerIdKind::Stable, "stable-A", &first))
        .unwrap();
    let hit = db
        .read(|tx| tx.find_by_server_id(&acc, ServerIdKind::Stable, "stable-A"))
        .unwrap();
    assert_eq!(hit, Some(first));
    let other_kind = db
        .read(|tx| tx.find_by_server_id(&acc, ServerIdKind::Uid, "stable-A"))
        .unwrap();
    let other_account = db
        .read(|tx| {
            tx.find_by_server_id(
                &AccountId::from_raw("acc-2"),
                ServerIdKind::Stable,
                "stable-A",
            )
        })
        .unwrap();
    assert_eq!((other_kind, other_account), (None, None));
}

/// Remapper la même valeur la rattache au nouveau message
#[test]
fn remapping_a_server_id_replaces_the_target() {
    let db = database_with_messages(2);
    let acc = AccountId::from_raw("acc-1");
    db.write(|tx| {
        tx.map_server_id(
            &acc,
            ServerIdKind::Stable,
            "k",
            &MessageId::from_raw("msg-1"),
        )?;
        tx.map_server_id(
            &acc,
            ServerIdKind::Stable,
            "k",
            &MessageId::from_raw("msg-2"),
        )
    })
    .unwrap();
    let hit = db
        .read(|tx| tx.find_by_server_id(&acc, ServerIdKind::Stable, "k"))
        .unwrap();
    assert_eq!(hit, Some(MessageId::from_raw("msg-2")));
}

/// Oublier les identifiants d'UID d'une boîte épargne les stables et les autres boîtes
#[test]
fn forgetting_uid_ids_keeps_stable_and_other_mailboxes() {
    let db = database_with_messages(1);
    let acc = AccountId::from_raw("acc-1");
    let msg = MessageId::from_raw("msg-1");
    db.write(|tx| {
        tx.map_server_id(&acc, ServerIdKind::Uid, "box-1/77/3", &msg)?;
        tx.map_server_id(&acc, ServerIdKind::Uid, "box-10/77/3", &msg)?;
        tx.map_server_id(&acc, ServerIdKind::Stable, "box-1/77/3", &msg)?;
        tx.forget_uid_ids_for_mailbox(&MailboxId::from_raw("box-1"))
    })
    .unwrap();
    let find = |kind, value: &str| {
        db.read(|tx| tx.find_by_server_id(&acc, kind, value))
            .unwrap()
    };
    assert_eq!(find(ServerIdKind::Uid, "box-1/77/3"), None);
    assert_eq!(find(ServerIdKind::Uid, "box-10/77/3"), Some(msg.clone()));
    assert_eq!(find(ServerIdKind::Stable, "box-1/77/3"), Some(msg));
}

/// L'empreinte rend tous les candidats (même Message-ID, date et taille) et l'appelant décide
#[test]
fn fingerprint_returns_every_candidate() {
    let db = database_with_messages(0);
    let acc = AccountId::from_raw("acc-1");
    db.write(|tx| {
        for name in ["twin-a", "twin-b", "other"] {
            let mut message = sample_message(name, "acc-1");
            message.message_id_header = Some("<same@example.org>".to_string());
            message.size = if name == "other" { 99 } else { 1_234 };
            tx.insert_message(&message, Timestamp::from_millis(1))?;
        }
        Ok(())
    })
    .unwrap();
    let date = Some(Timestamp::from_millis(5_000));
    let found = db
        .read(|tx| tx.find_by_fingerprint(&acc, "<same@example.org>", date, 1_234))
        .unwrap();
    let names: Vec<&str> = found.iter().map(|id| id.as_str()).collect();
    assert_eq!(names, ["twin-a", "twin-b"]);
}
