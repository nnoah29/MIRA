#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Préchargement des corps : un message récalcitrant ou énorme ne bloque jamais la synchronisation.

mod support_sync;

use mira_core::sync::{SyncSession, sync_account};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ContentState, PageRequest};
use support_sync::mail::mail;
use support_sync::mirror::mailbox_named;
use support_sync::provider::SyncTestProvider;
use support_sync::within;
use support_sync::world::{SyncWorld, open_session, start_world};

const OVERSIZE_BYTES: usize = 11 * 1024 * 1024;

/// Règle le nombre de corps à précharger dans le compte du miroir
fn set_prefetch(world: &SyncWorld, count: u32) {
    let id = world.ctx.account.id.clone();
    world
        .db
        .write(|tx| {
            let mut account = tx.get_account(&id)?.unwrap();
            account.sync.prefetch_bodies = count;
            tx.update_account(&account)
        })
        .unwrap();
}

/// Nombre de messages de la réception dont le corps est présent
fn bodies_present(world: &SyncWorld) -> usize {
    let inbox = mailbox_named(&world.db, "INBOX");
    let request = PageRequest {
        cursor: None,
        limit: 100,
    };
    let page = world
        .db
        .read(|tx| tx.list_messages_in_mailbox(&inbox.id, &request))
        .unwrap();
    let present = |m: &&mira_vocabulary::MessageSummary| m.content == ContentState::BodyPresent;
    page.items.iter().filter(present).count()
}

/// Monde dont la réception a trois courriers, déjà synchronisée sans aucun corps
async fn headers_only_world(extra: Option<Vec<u8>>) -> (SyncWorld, SyncSession) {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    for index in 1..=3 {
        let raw = mail(&format!("Courrier {index}"), &format!("c{index}@x"));
        world.store.deliver("alice", "INBOX", &raw, &[]);
    }
    if let Some(raw) = extra {
        world.store.deliver("alice", "INBOX", &raw, &[]);
    }
    set_prefetch(&world, 0);
    let mut session = open_session(&world).await;
    sync_account(&mut session).await.unwrap();
    assert_eq!(bodies_present(&world), 0);
    (world, session)
}

/// Un message dont le téléchargement est refusé est journalisé et ignoré : la passe réussit, les autres corps arrivent
#[tokio::test]
async fn regression_refused_body_prefetch_does_not_fail_the_pass() {
    within(async {
        let (world, mut session) = headers_only_world(None).await;
        set_prefetch(&world, 50);
        world.store.fail_next("UID FETCH", "NO [SERVERBUG] panne");
        sync_account(&mut session).await.unwrap();
        assert_eq!(bodies_present(&world), 2);
    })
    .await;
}

/// Un message de plus de 10 Mo n'est pas préchargé : il reste en en-têtes seuls, les autres corps arrivent
#[tokio::test]
async fn regression_oversized_message_is_not_prefetched() {
    within(async {
        let head = "From: Bob <bob@example.org>\r\nSubject: Énorme\r\nMessage-ID: <enorme@x>\r\nDate: Mon, 01 Jan 2024 10:00:00 +0000\r\n\r\n";
        let mut huge = head.as_bytes().to_vec();
        huge.extend(std::iter::repeat_n(b'a', OVERSIZE_BYTES));
        let (world, mut session) = headers_only_world(Some(huge)).await;
        set_prefetch(&world, 50);
        sync_account(&mut session).await.unwrap();
        assert_eq!(bodies_present(&world), 3);
    })
    .await;
}
