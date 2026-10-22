#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Reprise des règles : l'intégration d'un lot inscrit en attente les messages créés en direct, et eux seuls.

mod support_sync;

use mira_core::imap::FetchedHeaders;
use mira_core::sync::{Announce, store_batch, sync_account};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{Mailbox, MailboxRole, MessageId};
use support_sync::mail::mail;
use support_sync::mirror::account;
use support_sync::provider::SyncTestProvider;
use support_sync::within;
use support_sync::world::{SyncWorld, deliver_many, open_session, start_world};

/// Monde synchronisé une fois, et sa boîte de réception
async fn synced_inbox() -> (SyncWorld, Mailbox) {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    deliver_many(&world, "INBOX", "Initial", 1);
    let mut session = open_session(&world).await;
    sync_account(&mut session).await.unwrap();
    let inbox = world
        .db
        .read(|tx| tx.find_mailbox_by_role(&account(), MailboxRole::Inbox))
        .unwrap()
        .unwrap();
    (world, inbox)
}

/// En-têtes d'un nouveau message du serveur, d'UID et d'identifiant de message uniques
fn headers(uid: u32) -> FetchedHeaders {
    let raw = mail(&format!("Lot {uid}"), &format!("lot{uid}@x"));
    FetchedHeaders {
        uid,
        flags: Vec::new(),
        size: raw.len() as u64,
        internal_date: None,
        modseq: None,
        header_bytes: raw,
        extra: Default::default(),
    }
}

/// Intègre deux nouveaux messages dans la boîte et rend les identifiants créés
fn store_two(world: &SyncWorld, mailbox: &Mailbox, first: u32, mode: Announce) -> Vec<MessageId> {
    let validity = mailbox.marker.as_ref().map_or(1, |m| m.uid_validity);
    let fetched = vec![headers(first), headers(first + 1)];
    let outcomes = store_batch(&world.ctx, mailbox, validity, fetched, mode).unwrap();
    let created: Vec<MessageId> = outcomes
        .iter()
        .filter_map(|o| o.created_id())
        .cloned()
        .collect();
    assert_eq!(created.len(), 2, "deux messages créés");
    created
}

/// En direct, chaque message créé est inscrit en attente dans la même transaction
#[tokio::test]
async fn live_batch_records_every_created_message_as_pending() {
    within(async {
        let (world, inbox) = synced_inbox().await;
        let created = store_two(&world, &inbox, 500, Announce::Live);
        assert_eq!(world.db.read(|tx| tx.list_pending()).unwrap(), created);
    })
    .await;
}

/// En mode silencieux, rien n'est inscrit en attente
#[tokio::test]
async fn silent_batch_records_nothing() {
    within(async {
        let (world, inbox) = synced_inbox().await;
        store_two(&world, &inbox, 600, Announce::Silent);
        assert!(world.db.read(|tx| tx.list_pending()).unwrap().is_empty());
    })
    .await;
}

/// Dans une boîte exclue (Envoyés), rien n'est inscrit en attente, même en direct
#[tokio::test]
async fn excluded_role_records_nothing() {
    within(async {
        let (world, inbox) = synced_inbox().await;
        let sent = Mailbox {
            role: MailboxRole::Sent,
            ..inbox
        };
        store_two(&world, &sent, 700, Announce::Live);
        assert!(world.db.read(|tx| tx.list_pending()).unwrap().is_empty());
    })
    .await;
}
