#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rétention des événements : au plus 30 jours et 100 000 lignes, sans jamais réutiliser un numéro.

use mira_core::storage::Database;
use mira_vocabulary::{EventKind, Timestamp};

const DAY: i64 = 86_400_000;

/// Base en mémoire contenant `count` événements datés des instants donnés par `at`
fn database_with(count: i64, at: impl Fn(i64) -> i64) -> Database {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| {
        for index in 0..count {
            let when = Timestamp::from_millis(at(index));
            tx.append_event(when, &EventKind::EngineStopping)?;
        }
        Ok(())
    })
    .unwrap();
    db
}

/// Les événements de plus de 30 jours sont supprimés, les plus récents restent
#[test]
fn retention_drops_events_older_than_thirty_days() {
    let now = 100 * DAY;
    let db = database_with(6, |index| if index < 4 { 10 * DAY } else { 90 * DAY });
    let removed = db
        .write(|tx| tx.apply_event_retention(Timestamp::from_millis(now)))
        .unwrap();
    assert_eq!(removed, 4);
    let kept = db.read(|tx| tx.events_after(None, 100)).unwrap();
    assert_eq!(kept.len(), 2);
}

/// Au-delà de 100 000 lignes, seuls les derniers événements restent, même s'ils sont récents
#[test]
fn retention_caps_the_number_of_rows() {
    let now = 100 * DAY;
    let db = database_with(100_003, |_| 99 * DAY);
    let removed = db
        .write(|tx| tx.apply_event_retention(Timestamp::from_millis(now)))
        .unwrap();
    assert_eq!(removed, 3);
    let kept = db.read(|tx| tx.events_after(None, 200_000)).unwrap();
    assert_eq!(kept.len(), 100_000);
    assert_eq!(kept[0].seq.0, 4);
}

/// Après une purge totale, les nouveaux événements gardent des numéros croissants
#[test]
fn retention_never_reuses_event_numbers() {
    let now = 100 * DAY;
    let db = database_with(3, |_| DAY);
    db.write(|tx| tx.apply_event_retention(Timestamp::from_millis(now)))
        .unwrap();
    let next = db
        .write(|tx| tx.append_event(Timestamp::from_millis(now), &EventKind::EngineStarted))
        .unwrap();
    assert_eq!(next.seq.0, 4);
}
