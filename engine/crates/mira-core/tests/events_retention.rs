#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le moteur applique la rétention des événements à l'ouverture du profil.

mod support_api;

use mira_core::api::Engine;
use mira_core::storage::Database;
use mira_vocabulary::{CONTRACT_VERSION, EventKind, Timestamp};
use support_api::fixture;

const RECENT_MILLIS: i64 = 1_699_999_000_000;

/// À l'ouverture, les événements de plus de 30 jours disparaissent et les récents restent
#[tokio::test]
async fn regression_engine_prunes_expired_events_when_it_opens() {
    let f = fixture::start().await;
    let path = f.home.path().join("profiles/default/mira.sqlite3");
    f.engine.shutdown().unwrap();
    let database = Database::open(&path).unwrap();
    database
        .write(|tx| {
            tx.append_event(Timestamp::from_millis(1), &EventKind::EngineStopping)?;
            tx.append_event(
                Timestamp::from_millis(RECENT_MILLIS),
                &EventKind::EngineStopping,
            )?;
            Ok(())
        })
        .unwrap();
    drop(database);
    let engine = Engine::open(f.config.clone(), CONTRACT_VERSION).unwrap();
    let database = Database::open(&path).unwrap();
    let kept = database.read(|tx| tx.events_after(None, 1_000)).unwrap();
    assert!(kept.iter().all(|event| event.at.as_millis() > 1));
    assert!(
        kept.iter()
            .any(|event| event.at.as_millis() == RECENT_MILLIS)
    );
    engine.shutdown().unwrap();
}
