//! Le moteur s'ouvre sur un profil de test, répond au statut et se ferme sans blocage.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::time::Duration;

use mira_assembly::api::{EngineRequest, Request, Response};
use support::world::TestWorld;

/// Le moteur ouvert sur un profil vide répond au statut sans compte, puis se ferme.
#[tokio::test]
async fn opens_and_closes_engine_on_test_profile() {
    let world = TestWorld::new();
    let engine = world.engine();
    let response = engine
        .execute(Request::Engine(EngineRequest::GetStatus))
        .await;
    assert!(matches!(response, Ok(Response::Status(ref status)) if status.accounts.is_empty()));
    drop(engine);
    let closing = world.close();
    assert!(
        tokio::time::timeout(Duration::from_secs(10), closing)
            .await
            .is_ok()
    );
}
