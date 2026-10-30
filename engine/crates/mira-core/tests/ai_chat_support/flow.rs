//! Lecture complète d'un flux de génération et attentes bornées sur le faux serveur et l'opération.

use std::time::Duration;

use mira_core::api::{ChatStream, Engine, OperationRequest, Request, Response};
use mira_core::operations::OperationStatus;
use mira_testkit::ai::FakeAiServer;
use mira_vocabulary::{ChatDelta, OperationId};
use tokio::time::{Instant, sleep, timeout};

const LIMIT: Duration = Duration::from_secs(10);

/// Lit tous les deltas jusqu'à la fin du flux (10 s au plus)
pub async fn collect(stream: &mut ChatStream) -> Vec<ChatDelta> {
    let mut deltas = Vec::new();
    let read_all = async {
        while let Some(delta) = stream.next().await {
            deltas.push(delta);
        }
    };
    timeout(LIMIT, read_all).await.unwrap();
    deltas
}

/// Statut courant d'une opération
pub async fn operation_status(engine: &Engine, operation: &OperationId) -> OperationStatus {
    let request = Request::Operation(OperationRequest::GetOperation {
        operation: operation.clone(),
    });
    match engine.execute(request).await.unwrap() {
        Response::Operation(info) => info.status,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Attend que l'opération atteigne ce statut avant `limit` ; rend vrai si c'est le cas
pub async fn wait_status(
    engine: &Engine,
    operation: &OperationId,
    expected: &OperationStatus,
    limit: Duration,
) -> bool {
    let deadline = Instant::now() + limit;
    while Instant::now() < deadline {
        if operation_status(engine, operation).await == *expected {
            return true;
        }
        sleep(Duration::from_millis(10)).await;
    }
    false
}

/// Attend qu'au moins `count` connexions aient été fermées par le client (10 s au plus)
pub async fn wait_closed(server: &FakeAiServer, count: usize) -> bool {
    let deadline = Instant::now() + LIMIT;
    while Instant::now() < deadline {
        if server.connections_closed_by_client() >= count {
            return true;
        }
        sleep(Duration::from_millis(20)).await;
    }
    false
}
