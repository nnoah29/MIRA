//! Suivi des opérations longues par la façade : état courant et attente de la fin.

use std::time::Duration;

use mira_core::api::{Engine, OperationRequest, Request, Response};
use mira_core::operations::OperationStatus;
use mira_vocabulary::OperationId;

/// État courant d'une opération, lu par la façade
pub async fn status_of(engine: &Engine, operation: &OperationId) -> OperationStatus {
    let request = Request::Operation(OperationRequest::GetOperation {
        operation: operation.clone(),
    });
    match engine.execute(request).await.unwrap() {
        Response::Operation(info) => info.status,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Attend la fin d'une opération (au plus 10 secondes) et rend son état final
pub async fn wait_finished(engine: &Engine, operation: &OperationId) -> OperationStatus {
    let wait = async {
        loop {
            let status = status_of(engine, operation).await;
            if !matches!(status, OperationStatus::Running { .. }) {
                return status;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    };
    tokio::time::timeout(Duration::from_secs(10), wait)
        .await
        .unwrap()
}
