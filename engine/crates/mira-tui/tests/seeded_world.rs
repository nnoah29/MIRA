//! Le monde de test garni : compte IMAP sur le faux serveur et boîte amorcée visible par le moteur.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use mira_assembly::api::{MailboxRequest, Request, Response};
use mira_assembly::mira_vocabulary::MailboxRole;
use support::world::TestWorld;

/// Une boîte amorcée dans le miroir est listée par le moteur avec ses messages.
#[tokio::test]
async fn seeded_mailbox_is_visible_through_the_engine() {
    let world = TestWorld::with_imap_account().await;
    let account = world.account().unwrap();
    world
        .seed_mailbox("INBOX", MailboxRole::Inbox, &["Premier", "Second"])
        .unwrap();
    let engine = world.engine();
    let response = engine
        .execute(Request::Mailbox(MailboxRequest::ListMailboxes { account }))
        .await
        .unwrap();
    let listed = matches!(&response, Response::Mailboxes(m) if m.len() == 1 && m[0].total == 2);
    assert!(listed);
    drop(engine);
    world.close().await.unwrap();
}
