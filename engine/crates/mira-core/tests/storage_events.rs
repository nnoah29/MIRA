#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Événements numérotés : numérotation, relecture, rétention.

use mira_core::storage::Database;
use mira_vocabulary::{AccountId, EventKind, EventSeq, Timestamp};

/// Événement de test rattaché à un compte
fn account_event(name: &str) -> EventKind {
    EventKind::AccountAdded {
        account: AccountId::from_raw(name),
    }
}

/// Les numéros sont strictement croissants et l'événement rendu porte son numéro
#[test]
fn events_are_numbered_increasingly() {
    let db = Database::open_in_memory().unwrap();
    let seqs: Vec<u64> = db
        .write(|tx| {
            let first = tx.append_event(Timestamp::from_millis(1), &EventKind::EngineStarted)?;
            let second = tx.append_event(Timestamp::from_millis(2), &account_event("a"))?;
            Ok(vec![first.seq.0, second.seq.0])
        })
        .unwrap();
    assert_eq!(seqs, [1, 2]);
}

/// La relecture après un numéro rend les suivants dans l'ordre, bornés par la limite
#[test]
fn events_after_returns_later_events_in_order() {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| {
        for index in 0..5 {
            tx.append_event(
                Timestamp::from_millis(index),
                &account_event(&index.to_string()),
            )?;
        }
        Ok(())
    })
    .unwrap();
    let all = db.read(|tx| tx.events_after(None, 100)).unwrap();
    assert_eq!(all.len(), 5);
    let tail = db
        .read(|tx| tx.events_after(Some(EventSeq(2)), 100))
        .unwrap();
    let seqs: Vec<u64> = tail.iter().map(|event| event.seq.0).collect();
    assert_eq!(seqs, [3, 4, 5]);
    let limited = db.read(|tx| tx.events_after(Some(EventSeq(2)), 2)).unwrap();
    assert_eq!(limited.len(), 2);
    assert_eq!(limited[0], all[2]);
}

/// Les événements survivent à la réouverture et la numérotation continue
#[test]
fn events_persist_and_numbering_continues_after_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    let kind = account_event("persisted");
    let written = {
        let db = Database::open(&path).unwrap();
        db.write(|tx| tx.append_event(Timestamp::from_millis(7), &kind))
            .unwrap()
    };
    let db = Database::open(&path).unwrap();
    assert_eq!(db.read(|tx| tx.events_after(None, 10)).unwrap(), [written]);
    let next = db
        .write(|tx| tx.append_event(Timestamp::from_millis(8), &kind))
        .unwrap();
    assert_eq!(next.seq, EventSeq(2));
}

/// L'élagage garde les derniers événements et un numéro supprimé n'est jamais réutilisé
#[test]
fn prune_keeps_latest_and_never_reuses_numbers() {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| {
        for index in 0..6 {
            tx.append_event(Timestamp::from_millis(index), &account_event("x"))?;
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(db.write(|tx| tx.prune_events(2)).unwrap(), 4);
    let kept: Vec<u64> = db
        .read(|tx| tx.events_after(None, 10))
        .unwrap()
        .iter()
        .map(|event| event.seq.0)
        .collect();
    assert_eq!(kept, [5, 6]);
    assert_eq!(db.write(|tx| tx.prune_events(10)).unwrap(), 0);
    db.write(|tx| tx.prune_events(0)).unwrap();
    let after = db
        .write(|tx| tx.append_event(Timestamp::from_millis(9), &EventKind::EngineStopping))
        .unwrap();
    assert_eq!(after.seq, EventSeq(7));
}
