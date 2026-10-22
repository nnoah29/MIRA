#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Libellés locaux et règles à l'arrivée des messages : pose sur la conversation, héritage dans un fil.

mod support_sync;

use mira_core::sync::{SyncSession, sync_account};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{
    Condition, ConditionGroup, ConditionTest, EventKind, LocalLabel, LocalLabelId, Rule,
    RuleAction, RuleId, RuleScope, ThreadId,
};
use support_sync::mail::{mail, reply};
use support_sync::mirror::account;
use support_sync::provider::SyncTestProvider;
use support_sync::rules::{catch_all, mark_read_count, store_rule};
use support_sync::waiting::count_events;
use support_sync::within;
use support_sync::world::{SyncWorld, deliver_many, open_session, start_world};

/// Libellé « Urgent » enregistré dans la base du monde
fn store_label(world: &SyncWorld) -> LocalLabelId {
    let label = LocalLabel {
        id: LocalLabelId::from_raw("lab-1"),
        name: "Urgent".to_string(),
        color: 0xFFE8_542B,
    };
    world.db.write(|tx| tx.insert_local_label(&label)).unwrap();
    label.id
}

/// Monde synchronisé une fois (deux messages en réception) avec ces règles déjà enregistrées
async fn world_with(rules: &[Rule]) -> (SyncWorld, SyncSession) {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    for name in ["Sent", "Trash", "Junk", "Drafts"] {
        world.store.create_mailbox("alice", name, None);
    }
    store_label(&world);
    rules.iter().for_each(|rule| store_rule(&world, rule));
    deliver_many(&world, "INBOX", "Initial", 2);
    let mut session = open_session(&world).await;
    sync_account(&mut session).await.unwrap();
    (world, session)
}

/// Un nouveau message déclenche la pose du libellé sur sa conversation, avec un seul événement ; le téléchargement initial, rien
#[tokio::test]
async fn live_arrival_puts_the_label_on_the_new_conversation() {
    within(async {
        let label = LocalLabelId::from_raw("lab-1");
        let rule = catch_all(
            "r1",
            vec![RuleAction::AddLocalLabel {
                label: label.clone(),
            }],
        );
        let (world, mut session) = world_with(&[rule]).await;
        assert!(
            world
                .db
                .read(|tx| tx.list_label_assignments())
                .unwrap()
                .is_empty()
        );
        world
            .store
            .deliver("alice", "INBOX", &mail("Nouveau", "n1@x"), &[]);
        sync_account(&mut session).await.unwrap();
        let listed = world.db.read(|tx| tx.list_label_assignments()).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].labels, vec![label]);
        let changed = |k: &EventKind| matches!(k, EventKind::LocalLabelsChanged);
        assert_eq!(count_events(&world.recording, changed), 1);
    })
    .await;
}

/// Fil du message qui porte cet en-tête Message-ID
fn thread_of_header(world: &SyncWorld, header: &str) -> ThreadId {
    let batch = world
        .db
        .read(|tx| tx.messages_for_threading(&account(), None, 500))
        .unwrap();
    let found = batch.into_iter().find(|(_, _, m)| {
        let stored = m.message_id_header.as_deref().unwrap_or_default();
        stored.trim_matches(['<', '>']) == header
    });
    found.unwrap().2.thread.unwrap()
}

/// Règle « a le libellé → marquer lu »
fn reads_when_labeled(label: &LocalLabelId) -> Rule {
    let test = ConditionTest::HasLocalLabel {
        label: label.clone(),
    };
    Rule {
        id: RuleId::from_raw("r2"),
        name: "Étiquetés lus".to_string(),
        enabled: true,
        scope: RuleScope::AllAccounts,
        alternatives: vec![ConditionGroup {
            conditions: vec![Condition {
                negated: false,
                test,
            }],
        }],
        actions: vec![RuleAction::MarkRead { read: true }],
        schema_version: 1,
    }
}

/// Une réponse qui arrive dans une conversation déjà étiquetée est vue comme porteuse du libellé
#[tokio::test]
async fn a_reply_inherits_the_label_of_its_conversation() {
    within(async {
        let label = LocalLabelId::from_raw("lab-1");
        let (world, mut session) = world_with(&[reads_when_labeled(&label)]).await;
        world
            .store
            .deliver("alice", "INBOX", &mail("Plan", "p1@x"), &[]);
        sync_account(&mut session).await.unwrap();
        assert_eq!(mark_read_count(&world), 0);
        let thread = thread_of_header(&world, "p1@x");
        world
            .db
            .write(|tx| tx.set_label_on_thread(&label, &thread, true))
            .unwrap();
        world
            .store
            .deliver("alice", "INBOX", &reply("Plan", "p2@x", "p1@x"), &[]);
        sync_account(&mut session).await.unwrap();
        assert_eq!(mark_read_count(&world), 1);
    })
    .await;
}
