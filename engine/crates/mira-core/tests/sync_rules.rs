#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règles à l'arrivée des messages : synchronisation incrémentale seulement, boîtes exclues, silence.

mod support_sync;

use mira_core::sync::{SyncSession, sync_account};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::RuleAction;
use support_sync::mail::mail;
use support_sync::mirror::mailbox_named;
use support_sync::provider::SyncTestProvider;
use support_sync::rules::{
    catch_all, mark_read_count, queued, rule_applied, silenced_flags, store_rule,
};
use support_sync::within;
use support_sync::world::{SyncWorld, deliver_many, open_session, start_world};

/// Monde synchronisé une fois (deux messages en réception) avec une règle « tout » déjà enregistrée
async fn world_with_rule(actions: Vec<RuleAction>) -> (SyncWorld, SyncSession) {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    for name in ["Sent", "Trash", "Junk", "Drafts", "Work"] {
        world.store.create_mailbox("alice", name, None);
    }
    store_rule(&world, &catch_all("r1", actions));
    deliver_many(&world, "INBOX", "Initial", 2);
    let mut session = open_session(&world).await;
    sync_account(&mut session).await.unwrap();
    (world, session)
}

/// Un nouveau message arrivé en synchronisation incrémentale déclenche l'action et l'événement ; le téléchargement initial, rien
#[tokio::test]
async fn live_arrival_applies_the_rule_but_first_download_does_not() {
    within(async {
        let (world, mut session) = world_with_rule(vec![RuleAction::MarkRead { read: true }]).await;
        assert!(queued(&world).is_empty(), "téléchargement initial");
        let raw = mail("Nouveau", "n1@x");
        world.store.deliver("alice", "INBOX", &raw, &[]);
        sync_account(&mut session).await.unwrap();
        assert_eq!(mark_read_count(&world), 1);
        assert_eq!(rule_applied(&world), 1);
        assert_eq!(silenced_flags(&world), [false]);
        assert!(world.db.read(|tx| tx.list_pending()).unwrap().is_empty());
    })
    .await;
}

/// Une resynchronisation de boîte (UIDVALIDITY changé) n'applique aucune règle
#[tokio::test]
async fn resynchronization_applies_no_rule() {
    within(async {
        let (world, mut session) = world_with_rule(vec![RuleAction::MarkRead { read: true }]).await;
        let raw = mail("Pendant", "p1@x");
        world.store.deliver("alice", "INBOX", &raw, &[]);
        world.store.expunge("alice", "INBOX", 1);
        world.store.renumber("alice", "INBOX");
        sync_account(&mut session).await.unwrap();
        assert!(queued(&world).is_empty());
        assert_eq!(rule_applied(&world), 0);
    })
    .await;
}

/// Envoyés, Spam, Corbeille et Brouillons ne déclenchent jamais de règle
#[tokio::test]
async fn excluded_mailboxes_never_trigger_rules() {
    within(async {
        let (world, mut session) = world_with_rule(vec![RuleAction::MarkRead { read: true }]).await;
        for (n, name) in ["Sent", "Trash", "Junk", "Drafts"].iter().enumerate() {
            let raw = mail(&format!("Exclu {name}"), &format!("e{n}@x"));
            world.store.deliver("alice", name, &raw, &[]);
        }
        sync_account(&mut session).await.unwrap();
        assert_eq!(mailbox_named(&world.db, "Junk").total, 1);
        assert!(queued(&world).is_empty());
        assert_eq!(rule_applied(&world), 0);
    })
    .await;
}

/// Le silence demandé par une règle marque l'événement de nouveau message
#[tokio::test]
async fn suppress_notification_silences_the_new_message_event() {
    within(async {
        let (world, mut session) = world_with_rule(vec![RuleAction::SuppressNotification]).await;
        let raw = mail("Muet", "m1@x");
        world.store.deliver("alice", "INBOX", &raw, &[]);
        sync_account(&mut session).await.unwrap();
        assert_eq!(silenced_flags(&world), [true]);
        assert!(queued(&world).is_empty());
    })
    .await;
}
