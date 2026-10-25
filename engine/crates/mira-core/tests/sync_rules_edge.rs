#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règles à l'arrivée : règle invalide ignorée, message disparu avant traitement.

mod support_sync;

use mira_core::sync::{apply_rules_to_message, sync_account};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{MailboxId, MessageId, RuleAction};
use support_sync::mail::mail;
use support_sync::provider::SyncTestProvider;
use support_sync::rules::{catch_all, queued, rule_applied, store_rule};
use support_sync::within;
use support_sync::world::{deliver_many, open_session, start_world};

/// Une règle visant une boîte disparue est invalide : jamais exécutée
#[tokio::test]
async fn invalid_rule_is_ignored() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        let missing = MailboxId::from_raw("boite-disparue");
        let moved = vec![RuleAction::MoveTo { mailbox: missing }];
        store_rule(&world, &catch_all("r-bad", moved));
        deliver_many(&world, "INBOX", "Initial", 1);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let raw = mail("Nouveau", "n1@x");
        world.store.deliver("alice", "INBOX", &raw, &[]);
        sync_account(&mut session).await.unwrap();
        assert!(queued(&world).is_empty());
        assert_eq!(rule_applied(&world), 0);
        assert!(world.db.read(|tx| tx.list_pending()).unwrap().is_empty());
    })
    .await;
}

/// Un message présent dans deux boîtes (libellés multiples) n'est évalué qu'une fois
#[tokio::test]
async fn message_in_two_mailboxes_is_evaluated_once() {
    within(async {
        let caps = CapabilitySet::full().with_label_extensions();
        let world = start_world(caps, SyncTestProvider::with_stable_ids()).await;
        store_rule(
            &world,
            &catch_all("r1", vec![RuleAction::MarkRead { read: true }]),
        );
        deliver_many(&world, "INBOX", "Initial", 1);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let raw = mail("Double", "d1@x");
        let boxes = ["INBOX", "Travail"];
        world
            .store
            .deliver_with_labels("alice", &boxes, &raw, 555, 900);
        sync_account(&mut session).await.unwrap();
        assert_eq!(rule_applied(&world), 1);
        assert_eq!(queued(&world).len(), 1);
    })
    .await;
}

/// Un message disparu avant son traitement : la ligne en attente est retirée sans erreur
#[tokio::test]
async fn pending_row_of_a_vanished_message_is_removed_without_error() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        let ghost = MessageId::from_raw("fantome");
        world.db.write(|tx| tx.add_pending(&ghost)).unwrap();
        let outcome = apply_rules_to_message(&world.ctx, &ghost).unwrap();
        assert!(outcome.actions.is_empty() && !outcome.silenced);
        assert!(world.db.read(|tx| tx.list_pending()).unwrap().is_empty());
    })
    .await;
}
