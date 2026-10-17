#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! File d'actions : persistance, idempotence, ordre de rejeu, échecs.

mod support;

use mira_core::storage::Database;
use mira_vocabulary::{AccountId, ActionId, ActionState, IdempotencyKey, Timestamp};
use support::accounts::{database_with_account, sample_account};
use support::intentions::{sample_action, sample_move};

/// Invariant 17 : une action mise en file est relue après fermeture et réouverture de la base
#[test]
fn invariant_17_action_persisted_before_attempt() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    let action = sample_action("a-1", "acc-1", "key-1", 0);
    {
        let db = Database::open(&path).unwrap();
        db.write(|tx| {
            tx.insert_account(&sample_account("acc-1"))?;
            tx.enqueue_action(&action)
        })
        .unwrap();
    }
    let reopened = Database::open(&path).unwrap();
    let loaded = reopened.read(|tx| tx.get_action(&action.id)).unwrap();
    assert_eq!(loaded, Some(action));
}

/// Invariant 18 : la même clé pour le même compte ne crée qu'une action, la première
#[test]
fn invariant_18_enqueue_is_idempotent() {
    let db = database_with_account("acc-1");
    let first = sample_action("a-1", "acc-1", "same-key", 0);
    let second = sample_action("a-2", "acc-1", "same-key", 0);
    let stored_first = db.write(|tx| tx.enqueue_action(&first)).unwrap();
    let stored_second = db.write(|tx| tx.enqueue_action(&second)).unwrap();
    assert_eq!(stored_first.id.as_str(), "a-1");
    assert_eq!(stored_second.id.as_str(), "a-1");
    assert_eq!(db.read(|tx| tx.table_row_count("actions")).unwrap(), 1);
    let conflicting = sample_move("a-3", "acc-1", "same-key");
    let error = db.write(|tx| tx.enqueue_action(&conflicting)).unwrap_err();
    assert_eq!(error.code, "invalid_input.idempotency_conflict");
}

/// Invariant 19 : les actions sont rejouées dans l'ordre de mise en file
#[test]
fn invariant_19_actions_replayed_in_order() {
    let db = database_with_account("acc-1");
    let account = AccountId::from_raw("acc-1");
    db.write(|tx| {
        for (id, key) in [("c", "k3"), ("a", "k1"), ("b", "k2")] {
            tx.enqueue_action(&sample_action(id, "acc-1", key, 0))?;
        }
        Ok(())
    })
    .unwrap();
    let now = Timestamp::from_millis(10);
    let mut order = Vec::new();
    while let Some(next) = db.read(|tx| tx.next_ready_action(&account, now)).unwrap() {
        order.push(next.id.as_str().to_string());
        db.write(|tx| tx.set_action_state(&next.id, ActionState::Succeeded))
            .unwrap();
    }
    assert_eq!(order, ["c", "a", "b"]);
}

/// Une action dont la date d'essai n'est pas atteinte est sautée, puis reprise le moment venu
#[test]
fn next_ready_skips_actions_not_yet_due() {
    let db = database_with_account("acc-1");
    let account = AccountId::from_raw("acc-1");
    db.write(|tx| {
        tx.enqueue_action(&sample_action("late", "acc-1", "k1", 500))?;
        tx.enqueue_action(&sample_action("soon", "acc-1", "k2", 0))
    })
    .unwrap();
    let at = |millis| {
        db.read(|tx| tx.next_ready_action(&account, Timestamp::from_millis(millis)))
            .unwrap()
            .map(|action| action.id.into_string())
    };
    assert_eq!(at(100).as_deref(), Some("soon"));
    db.write(|tx| tx.set_action_state(&ActionId::from_raw("soon"), ActionState::InProgress))
        .unwrap();
    assert_eq!(at(100), None);
    assert_eq!(at(500).as_deref(), Some("late"));
}

/// Un échec compte une tentative, garde l'erreur et repousse l'essai
#[test]
fn failure_counts_an_attempt_and_delays_the_next_one() {
    let db = database_with_account("acc-1");
    let id = ActionId::from_raw("a-1");
    db.write(|tx| {
        tx.enqueue_action(&sample_action("a-1", "acc-1", "k1", 0))?;
        tx.record_action_failure(&id, "network.timeout", Timestamp::from_millis(900))?;
        tx.record_action_failure(&id, "network.timeout", Timestamp::from_millis(1_800))
    })
    .unwrap();
    let failed = db.read(|tx| tx.get_action(&id)).unwrap().unwrap();
    assert_eq!(failed.attempts, 2);
    assert_eq!(failed.last_error.as_deref(), Some("network.timeout"));
    assert_eq!(failed.not_before, Timestamp::from_millis(1_800));
    assert_eq!(failed.state, ActionState::Pending);
}

/// Les listes filtrent par état, la clé retrouve son action, un identifiant inconnu est refusé
#[test]
fn lists_filter_by_state_and_unknown_ids_are_refused() {
    let db = database_with_account("acc-1");
    let account = AccountId::from_raw("acc-1");
    let id = ActionId::from_raw("a-1");
    db.write(|tx| {
        tx.enqueue_action(&sample_action("a-1", "acc-1", "k1", 0))?;
        tx.enqueue_action(&sample_action("a-2", "acc-1", "k2", 0))?;
        tx.set_action_state(&ActionId::from_raw("a-2"), ActionState::Failed)
    })
    .unwrap();
    assert_eq!(db.read(|tx| tx.pending_action_count(&account)).unwrap(), 1);
    let only_failed = db
        .read(|tx| tx.list_actions(&account, &[ActionState::Failed]))
        .unwrap();
    assert_eq!(only_failed.len(), 1);
    assert_eq!(
        db.read(|tx| tx.list_actions(&account, &[])).unwrap().len(),
        2
    );
    let ghost = ActionId::from_raw("ghost");
    let error = db.write(|tx| tx.set_action_state(&ghost, ActionState::Failed));
    assert_eq!(error.unwrap_err().code, "not_found.action");
    let key = IdempotencyKey("k1".to_string());
    let found = db.read(|tx| tx.find_action_by_idempotency_key(&account, &key));
    assert_eq!(found.unwrap().unwrap().id, id);
}
