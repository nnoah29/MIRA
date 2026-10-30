//! Raccourcis des tests de génération : démarrage sur la boucle locale, attentes d'annulation et de requête.

use std::time::Duration;

use mira_core::api::{ChatStream, Engine, EventRequest, Request, Response};
use mira_core::operations::OperationStatus;
use mira_testkit::ai::{Behavior, FakeAiServer};
use mira_vocabulary::{EventFilter, EventKind, OperationId};
use tokio::time::sleep;

use super::connection::{add_chat_connection, chat_start};
use super::flow::wait_status;
use super::privacy::journal_rows;
use crate::ai_models_support::ModelsFixture;

/// Joue ce comportement et démarre une génération sur la connexion locale `conn-1` (modèle `model-a`)
pub async fn loopback_stream(
    fixture: &ModelsFixture,
    behavior: Behavior,
    text: &str,
) -> ChatStream {
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(fixture, "conn-1", base, Some("model-a"));
    fixture.server.queue(behavior);
    fixture
        .engine
        .chat_stream(chat_start(&connection, text))
        .await
        .unwrap()
}

/// Attend que l'opération soit annulée avant `limit`
pub async fn wait_cancelled(engine: &Engine, operation: &OperationId, limit: Duration) -> bool {
    wait_status(engine, operation, &OperationStatus::Cancelled, limit).await
}

/// Attend que le faux serveur ait reçu une requête (5 s au plus)
pub async fn wait_first_request(server: &FakeAiServer) {
    for _ in 0..500 {
        if !server.requests().is_empty() {
            return;
        }
        sleep(Duration::from_millis(10)).await;
    }
}

/// Attend que la ligne de journal de cette opération ne soit plus `running` (5 s au plus) et rend son issue
pub async fn wait_journal_outcome(fixture: &ModelsFixture, operation: &OperationId) -> String {
    for _ in 0..500 {
        let rows = journal_rows(fixture);
        let done = rows
            .iter()
            .find(|row| row.operation == *operation && row.outcome != "running");
        if let Some(row) = done {
            return row.outcome.clone();
        }
        sleep(Duration::from_millis(10)).await;
    }
    "running".to_string()
}

/// Genres de tous les événements du journal du moteur
pub async fn journal_kinds(engine: &Engine) -> Vec<EventKind> {
    let request = Request::Event(EventRequest::GetEvents {
        filter: EventFilter::default(),
        limit: 1000,
    });
    match engine.execute(request).await.unwrap() {
        Response::Events(events) => events.into_iter().map(|event| event.kind).collect(),
        other => panic!("réponse inattendue : {other:?}"),
    }
}
