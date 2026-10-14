#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Invariant 25 : toutes les connexions IMAP d'un compte (lectures explicites, tests de connexion) partagent un seul budget.

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use std::future::Future;
use std::time::Duration;

use mira_core::api::{AccountRequest, CredentialInput, Request, SearchRequest, SecretInput};
use mira_core::sync::ConnectionBudget;
use mira_testkit::imap::FakeImapServer;
use mira_vocabulary::AuthMethodKind;
use support_api::fixture;
use support_api::requests::draft;
use support_read::calls::{operation_of, run, wait_finished};
use support_read::world::start_world;

/// Plus grand nombre de connexions ouvertes pendant l'exécution du travail
async fn peak_while<F: Future>(imap: &FakeImapServer, work: F) -> usize {
    let mut work = Box::pin(work);
    let mut peak = 0;
    loop {
        tokio::select! {
            _ = &mut work => return peak,
            () = tokio::time::sleep(Duration::from_micros(300)) => {
                peak = peak.max(imap.active_connections());
            }
        }
    }
}

/// Douze recherches serveur simultanées n'ouvrent jamais plus de connexions que la limite du provider
#[tokio::test]
async fn regression_parallel_server_searches_stay_within_the_connection_budget() {
    let world = start_world().await;
    let engine = &world.fixture.engine;
    let work = async {
        let mut started = Vec::new();
        for _ in 0..12 {
            let request = Request::Search(SearchRequest::SearchServer {
                account: world.account.clone(),
                query: "licorne".to_string(),
            });
            started.push(operation_of(run(engine, request).await.unwrap()));
        }
        for operation in &started {
            wait_finished(engine, operation).await;
        }
    };
    let peak = tokio::time::timeout(
        Duration::from_secs(30),
        peak_while(&world.fixture.imap, work),
    )
    .await
    .unwrap();
    assert!(peak >= 1, "le serveur n'a jamais été sollicité");
    assert!(
        peak <= 3,
        "{peak} connexions simultanées pour un budget de 3"
    );
}

/// Un test de connexion pour la fixture, réussi
async fn connection_test(fixture: &fixture::Fixture) {
    let request = Request::Account(AccountRequest::TestConnection {
        draft: draft(fixture::FOLDERS_USER, AuthMethodKind::Password),
        credential: CredentialInput::Password(SecretInput::new(fixture::FOLDERS_PASSWORD)),
    });
    fixture.engine.execute(request).await.unwrap();
}

/// Huit tests de connexion simultanés pour la même adresse restent dans le budget du provider
#[tokio::test]
async fn regression_parallel_connection_tests_stay_within_the_connection_budget() {
    let fixture = fixture::start().await;
    let work = async {
        tokio::join!(
            connection_test(&fixture),
            connection_test(&fixture),
            connection_test(&fixture),
            connection_test(&fixture),
            connection_test(&fixture),
            connection_test(&fixture),
            connection_test(&fixture),
            connection_test(&fixture),
        );
    };
    let peak = tokio::time::timeout(Duration::from_secs(30), peak_while(&fixture.imap, work))
        .await
        .unwrap();
    assert!(peak >= 1, "le serveur n'a jamais été sollicité");
    assert!(
        peak <= 3,
        "{peak} connexions simultanées pour un budget de 3"
    );
}

/// Au-delà de seize demandes explicites en attente, la suivante est refusée au lieu de s'empiler
#[tokio::test]
async fn regression_explicit_requests_beyond_the_waiting_cap_are_refused() {
    let budget = ConnectionBudget::new(1);
    let held = budget.acquire().await.unwrap();
    let mut waiting = Vec::new();
    for _ in 0..16 {
        let queue = budget.clone();
        waiting.push(tokio::spawn(async move { queue.acquire_explicit().await }));
    }
    tokio::task::yield_now().await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let refused = budget.acquire_explicit().await.unwrap_err();
    assert_eq!(refused.code, "server.rate_limited");
    drop(held);
    for task in waiting {
        assert!(task.await.unwrap().is_ok());
    }
}
