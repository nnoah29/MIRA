#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Reprise des règles qui posent un libellé local : un message laissé en attente est repris sans doublon.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_sync;
mod support_write;

use mira_vocabulary::{EventKind, LocalLabel, LocalLabelId, MessageId, RuleAction};
use support_api::queries::event_kinds;
use support_imap::within;
use support_label_api::threads::seed_thread;
use support_sync::rules::catch_all;
use support_write::fixture::Fixture;
use support_write::waits::eventually;

/// Nombre de conversations qui portent un libellé
fn assigned(f: &Fixture) -> usize {
    f.db.read(|tx| tx.list_label_assignments()).unwrap().len()
}

/// Vrai quand plus aucun message n'attend d'évaluation
fn pending_is_empty(f: &Fixture) -> bool {
    f.db.read(|tx| tx.list_pending()).unwrap().is_empty()
}

/// Nombre d'événements « libellés locaux changés »
async fn changed(f: &Fixture) -> usize {
    let kinds = event_kinds(f.engine()).await;
    kinds
        .iter()
        .filter(|k| matches!(k, EventKind::LocalLabelsChanged))
        .count()
}

/// Banc avec le libellé Urgent, une règle qui le pose partout, et un message de fil laissé en attente
async fn crashed_with_pending() -> (Fixture, MessageId) {
    let f = Fixture::start().await;
    let (_, message) = seed_thread(&f.db, &f.account, "x");
    let id = LocalLabelId::from_raw("lab-1");
    let label = LocalLabel {
        id: id.clone(),
        name: "Urgent".to_string(),
        color: 1,
    };
    f.db.write(|tx| tx.insert_local_label(&label)).unwrap();
    let rule = catch_all("r1", vec![RuleAction::AddLocalLabel { label: id }]);
    f.db.write(|tx| tx.insert_rule(&rule)).unwrap();
    f.db.write(|tx| tx.add_pending(&message)).unwrap();
    (f, message)
}

/// Rejouer deux fois le même message en attente pose le libellé une fois et n'annonce qu'un changement
#[tokio::test]
async fn replaying_a_pending_message_twice_labels_once() {
    within(async {
        let (mut f, message) = crashed_with_pending().await;
        f.restart();
        eventually("pose du libellé", || assigned(&f) == 1).await;
        f.db.write(|tx| tx.add_pending(&message)).unwrap();
        f.restart();
        eventually("seconde reprise", || pending_is_empty(&f)).await;
        assert_eq!(assigned(&f), 1);
        assert_eq!(changed(&f).await, 1);
    })
    .await;
}
