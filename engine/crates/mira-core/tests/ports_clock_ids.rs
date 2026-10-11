#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Ports d'horloge et de génération d'identifiants.

use mira_core::ports::{Clock, IdGenerator, ManualClock, SequentialIds, SystemClock, UuidV7Ids};
use mira_vocabulary::Timestamp;

/// L'horloge manuelle reste immobile jusqu'à ce qu'on la règle ou l'avance
#[test]
fn manual_clock_moves_only_when_told() {
    let clock = ManualClock::new(Timestamp::from_millis(1_000));
    assert_eq!(clock.now(), Timestamp::from_millis(1_000));
    assert_eq!(clock.now(), Timestamp::from_millis(1_000));
    clock.advance_millis(250);
    assert_eq!(clock.now(), Timestamp::from_millis(1_250));
    clock.set(Timestamp::from_millis(5));
    assert_eq!(clock.now(), Timestamp::from_millis(5));
}

/// L'horloge système donne une date postérieure à 2020 et ne recule pas
#[test]
fn system_clock_is_plausible_and_monotone_enough() {
    let first = SystemClock.now();
    let second = SystemClock.now();
    assert!(first.as_millis() > 1_577_836_800_000);
    assert!(second >= first);
}

/// La séquence d'identifiants est déterministe et indépendante d'une instance à l'autre
#[test]
fn sequential_ids_are_deterministic() {
    let ids = SequentialIds::new();
    assert_eq!(ids.next_id(), "id-000001");
    assert_eq!(ids.next_id(), "id-000002");
    assert_eq!(SequentialIds::new().next_id(), "id-000001");
}

/// Les identifiants UUID v7 sont uniques et portent le numéro de version 7
#[test]
fn uuid_v7_ids_are_unique_and_versioned() {
    let ids = UuidV7Ids;
    let first = ids.next_id();
    let second = ids.next_id();
    assert_ne!(first, second);
    assert_eq!(first.len(), 36);
    assert_eq!(first.as_bytes()[14], b'7');
}

/// Les générateurs sont utilisables derrière un objet de trait partagé entre fils
#[test]
fn generators_work_as_shared_trait_objects() {
    let ids: std::sync::Arc<dyn IdGenerator> = std::sync::Arc::new(SequentialIds::new());
    let clone = ids.clone();
    let handle = std::thread::spawn(move || clone.next_id());
    let remote = handle.join().unwrap();
    assert_ne!(remote, ids.next_id());
}
