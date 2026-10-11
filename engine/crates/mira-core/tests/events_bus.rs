#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Bus d'événements : numérotation, persistance, rattrapage, direct, filtres (12.5, invariant 32).

use std::sync::Arc;
use std::time::Duration;

use mira_core::events::{EventBus, EventSink, RecordingSink};
use mira_core::ports::ManualClock;
use mira_core::storage::Database;
use mira_vocabulary::{AccountId, EventCategory, EventFilter, EventKind, EventSeq, Timestamp};

/// Construit un bus sur une base en mémoire
fn bus() -> EventBus {
    let clock = Arc::new(ManualClock::new(Timestamp::from_millis(1_000)));
    EventBus::new(Arc::new(Database::open_in_memory().unwrap()), clock)
}

/// Les événements émis sont numérotés dans l'ordre et relus depuis la base
#[test]
fn invariant_32_events_are_numbered_and_persisted() {
    let bus = bus();
    let first = bus.emit(EventKind::EngineStarted).unwrap();
    let second = bus.emit(EventKind::EngineStopping).unwrap();
    assert!(second.seq > first.seq);
    let replay = bus.catch_up(&EventFilter::default(), 10).unwrap();
    assert_eq!(
        replay.iter().map(|e| e.seq).collect::<Vec<_>>(),
        vec![first.seq, second.seq]
    );
}

/// Un client qui se reconnecte avec `after` reçoit seulement ce qu'il a manqué, puis le direct, sans doublon
#[tokio::test]
async fn invariant_32_subscription_resumes_after_seen_number_then_follows_live() {
    let bus = bus();
    let seen = bus.emit(EventKind::EngineStarted).unwrap();
    let missed = bus.emit(EventKind::EngineStopping).unwrap();
    let mut subscription = bus.subscribe(EventFilter {
        after: Some(seen.seq),
        ..EventFilter::default()
    });
    let live = bus
        .emit(EventKind::AccountAdded {
            account: AccountId::from_raw("acc-1"),
        })
        .unwrap();
    let wait = Duration::from_secs(2);
    let got_missed = tokio::time::timeout(wait, subscription.next())
        .await
        .unwrap()
        .unwrap();
    let got_live = tokio::time::timeout(wait, subscription.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!((got_missed.seq, got_live.seq), (missed.seq, live.seq));
    assert!(
        tokio::time::timeout(Duration::from_millis(100), subscription.next())
            .await
            .is_err()
    );
}

/// Le filtre par catégorie et par compte s'applique au rattrapage et au direct
#[tokio::test]
async fn subscription_applies_category_and_account_filters() {
    let bus = bus();
    let account = AccountId::from_raw("acc-1");
    bus.emit(EventKind::EngineStarted).unwrap();
    bus.emit(EventKind::AccountAdded {
        account: AccountId::from_raw("acc-2"),
    })
    .unwrap();
    let filter = EventFilter {
        accounts: vec![account.clone()],
        categories: vec![EventCategory::Account],
        after: None,
    };
    let mut subscription = bus.subscribe(filter);
    let wanted = bus.emit(EventKind::AccountAdded { account }).unwrap();
    let got = tokio::time::timeout(Duration::from_secs(2), subscription.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.seq, wanted.seq);
}

/// L'enregistreur de test numérote à partir de 1 et garde l'ordre
#[test]
fn recording_sink_keeps_order_and_numbers() {
    let sink = RecordingSink::default();
    sink.emit(EventKind::EngineStarted).unwrap();
    let second = sink.emit(EventKind::EngineStopping).unwrap();
    assert_eq!(second.seq, EventSeq(2));
    assert_eq!(
        sink.kinds(),
        vec![EventKind::EngineStarted, EventKind::EngineStopping]
    );
}
