#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Listes et lectures locales par la façade : boîtes, messages paginés, identifiants opaques (invariants 29 et 30).

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use mira_core::api::{MailboxRequest, Request, Response};
use mira_vocabulary::{Cursor, MailboxId};
use support_read::calls::{messages_page, page_of, run};
use support_read::seed::add_archive;
use support_read::world::start_world;

/// Les boîtes du compte sont rendues avec des compteurs cohérents avec les messages
#[tokio::test]
async fn list_mailboxes_reports_counters() {
    let world = start_world().await;
    world.seed_messages(3);
    let request = Request::Mailbox(MailboxRequest::ListMailboxes {
        account: world.account.clone(),
    });
    let Response::Mailboxes(boxes) = run(&world.fixture.engine, request).await.unwrap() else {
        panic!("réponse inattendue");
    };
    assert_eq!(boxes.len(), 1);
    assert_eq!((boxes[0].total, boxes[0].unread), (3, 3));
}

/// Le choix des boîtes synchronisées aligne le compte et l'indicateur de chaque boîte
#[tokio::test]
async fn set_synced_mailboxes_updates_selection_and_flags() {
    let world = start_world().await;
    add_archive(&world.mirror, &world.account);
    let request = world.select_synced(vec![world.mailbox.clone()]);
    let done = run(&world.fixture.engine, request).await;
    assert_eq!(done.unwrap(), Response::Done);
    let stored = world
        .mirror
        .read(|tx| tx.list_mailboxes(&world.account))
        .unwrap();
    let flags: Vec<(&str, bool)> = stored
        .iter()
        .map(|m| (m.name.as_str(), m.sync_enabled))
        .collect();
    assert_eq!(flags, vec![("Archive", false), ("INBOX", true)]);
}

/// Une boîte inconnue est refusée
#[tokio::test]
async fn set_synced_mailboxes_refuses_unknown_mailbox() {
    let world = start_world().await;
    let request = world.select_synced(vec![MailboxId::from_raw("nope")]);
    let refused = run(&world.fixture.engine, request).await.unwrap_err();
    assert_eq!(refused.code, "invalid_input.parameter");
}

/// Invariant 29 : une liste est découpée en pages bornées par curseur, sans doublon ni trou
#[tokio::test]
async fn invariant_29_lists_are_paginated_through_the_facade() {
    let world = start_world().await;
    world.seed_messages(5);
    let engine = &world.fixture.engine;
    let mut seen: Vec<String> = Vec::new();
    let mut cursor: Option<Cursor> = None;
    let mut sizes = Vec::new();
    loop {
        let page = page_of(
            messages_page(engine, &world.mailbox, 2, cursor)
                .await
                .unwrap(),
        );
        sizes.push(page.items.len());
        seen.extend(page.items.iter().map(|m| m.id.as_str().to_string()));
        cursor = page.next_cursor;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(sizes, vec![2, 2, 1]);
    assert_eq!(seen, vec!["m5", "m4", "m3", "m2", "m1"]);
    let bad = messages_page(engine, &world.mailbox, 2, Some(Cursor("zz".into()))).await;
    assert_eq!(bad.unwrap_err().code, "invalid_input.cursor");
}

/// Invariant 30 : les identifiants rendus par la façade sont des chaînes opaques
#[tokio::test]
async fn invariant_30_ids_are_opaque_strings() {
    let world = start_world().await;
    world.seed_messages(1);
    let response = messages_page(&world.fixture.engine, &world.mailbox, 10, None)
        .await
        .unwrap();
    let value = serde_json::to_value(&response).unwrap();
    let item = &value["data"]["items"][0];
    for field in ["id", "account"] {
        assert!(item[field].is_string(), "{field} doit être une chaîne");
    }
    assert!(item["mailboxes"][0].is_string());
}

/// Un message inconnu donne `not_found.message`
#[tokio::test]
async fn get_unknown_message_is_not_found() {
    let world = start_world().await;
    let request = Request::Message(mira_core::api::MessageRequest::GetMessage {
        message: mira_vocabulary::MessageId::from_raw("absent"),
    });
    let error = run(&world.fixture.engine, request).await.unwrap_err();
    assert_eq!(error.code, "not_found.message");
}
