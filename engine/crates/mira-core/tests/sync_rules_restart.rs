#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Reprise des règles au redémarrage : message laissé en attente par un arrêt brutal, rejeu idempotent, aucun secret.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_sync;
mod support_write;

use mira_vocabulary::{ActionKind, MessageId, RuleAction};
use support_api::queries::event_kinds;
use support_imap::within;
use support_sync::rules::catch_all;
use support_write::PASSWORD;
use support_write::fixture::Fixture;
use support_write::waits::eventually;

/// Banc dont une règle « tout marquer lu » est enregistrée et dont msg-1 est resté en attente d'évaluation
async fn crashed_with_pending() -> (Fixture, MessageId) {
    let f = Fixture::start().await;
    let rule = catch_all("r1", vec![RuleAction::MarkRead { read: true }]);
    f.db.write(|tx| tx.insert_rule(&rule)).unwrap();
    let message = MessageId::from_raw("msg-1");
    f.db.write(|tx| tx.add_pending(&message)).unwrap();
    (f, message)
}

/// Nombre d'actions « marquer lu » nées des règles, quel que soit leur état
fn rule_actions(f: &Fixture) -> usize {
    let all = f.db.read(|tx| tx.list_actions(&f.account, &[])).unwrap();
    all.iter()
        .filter(|a| matches!(a.kind, ActionKind::MarkRead { read: true }))
        .count()
}

/// Au redémarrage, un message resté en attente est évalué : une action, la ligne retirée
#[tokio::test]
async fn restart_replays_pending_rules_once() {
    within(async {
        let (mut f, _) = crashed_with_pending().await;
        f.restart();
        eventually("action de règle en file", || rule_actions(&f) == 1).await;
        assert!(f.db.read(|tx| tx.list_pending()).unwrap().is_empty());
    })
    .await;
}

/// Une reprise rejouée sur le même message n'ajoute aucune action
#[tokio::test]
async fn replaying_the_same_message_twice_adds_no_action() {
    within(async {
        let (mut f, message) = crashed_with_pending().await;
        f.restart();
        eventually("première reprise", || rule_actions(&f) == 1).await;
        f.db.write(|tx| tx.add_pending(&message)).unwrap();
        f.restart();
        eventually("seconde reprise", || {
            f.db.read(|tx| tx.list_pending()).unwrap().is_empty()
        })
        .await;
        assert_eq!(rule_actions(&f), 1);
    })
    .await;
}

/// Aucun secret dans les événements émis par la reprise
#[tokio::test]
async fn replay_events_carry_no_secret() {
    within(async {
        let (mut f, _) = crashed_with_pending().await;
        f.restart();
        eventually("action de règle en file", || rule_actions(&f) == 1).await;
        let kinds = event_kinds(f.engine()).await;
        assert!(format!("{kinds:?}").contains("RuleApplied"));
        assert!(!format!("{kinds:?}").contains(PASSWORD));
    })
    .await;
}
