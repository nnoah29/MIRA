#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{
    AccountId, Event, EventCategory, EventFilter, EventKind, EventSeq, Timestamp,
};

/// Fabrique une alerte de quota pleine pour `account` avec le numéro `seq`.
fn quota_full(seq: u64, account: &str) -> Event {
    Event {
        seq: EventSeq(seq),
        at: Timestamp::from_millis(0),
        kind: EventKind::QuotaFull {
            account: AccountId::from_raw(account),
        },
    }
}

/// Fabrique un événement de démarrage du moteur, sans compte, avec le numéro `seq`.
fn engine_started(seq: u64) -> Event {
    Event {
        seq: EventSeq(seq),
        at: Timestamp::from_millis(0),
        kind: EventKind::EngineStarted,
    }
}

/// Un filtre vide laisse tout passer, y compris les événements sans compte.
#[test]
fn empty_filter_matches_everything() {
    let filter = EventFilter::default();
    assert!(filter.matches(&quota_full(1, "a1")));
    assert!(filter.matches(&engine_started(2)));
}

/// Le filtre par compte ne garde que les événements des comptes listés.
#[test]
fn filter_by_account_keeps_only_listed_accounts() {
    let filter = EventFilter {
        accounts: vec![AccountId::from_raw("a1")],
        ..EventFilter::default()
    };
    assert!(filter.matches(&quota_full(1, "a1")));
    assert!(!filter.matches(&quota_full(2, "a2")));
    assert!(!filter.matches(&engine_started(3)));
}

/// Le filtre par catégorie ne garde que les événements de ces catégories.
#[test]
fn filter_by_category_keeps_only_listed_categories() {
    let filter = EventFilter {
        categories: vec![EventCategory::Alert],
        ..EventFilter::default()
    };
    assert!(filter.matches(&quota_full(1, "a1")));
    assert!(!filter.matches(&engine_started(2)));
}

/// Le filtre `after` exclut le numéro donné et tous les précédents.
#[test]
fn filter_after_excludes_given_sequence_and_older() {
    let filter = EventFilter {
        after: Some(EventSeq(10)),
        ..EventFilter::default()
    };
    assert!(!filter.matches(&engine_started(9)));
    assert!(!filter.matches(&engine_started(10)));
    assert!(filter.matches(&engine_started(11)));
}
