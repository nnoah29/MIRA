//! Appels de la façade pour la synchronisation : requêtes, état, attente d'événements et d'opérations.

use std::time::Duration;

use mira_core::api::{AccountSyncStatus, Engine, Request, Response, SyncRequest, SyncResponse};
use mira_core::operations::{OperationInfo, OperationStatus};
use mira_vocabulary::EventKind;

use super::queries::event_kinds;

/// Exécute une requête de synchronisation et rend sa réponse
pub async fn sync_call(engine: &Engine, request: SyncRequest) -> Response {
    engine.execute(Request::Sync(request)).await.unwrap()
}

/// Exécute une requête longue et rend la fiche de l'opération démarrée
pub async fn start_operation(engine: &Engine, request: SyncRequest) -> OperationInfo {
    match sync_call(engine, request).await {
        Response::Operation(info) => info,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// État de synchronisation de chaque compte
pub async fn sync_status(engine: &Engine) -> Vec<AccountSyncStatus> {
    match sync_call(engine, SyncRequest::GetSyncStatus).await {
        Response::Sync(SyncResponse::Status(list)) => list,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Attend (au plus 15 s) que les événements émis jusqu'ici satisfassent la condition, et les rend
pub async fn wait_for_events(
    engine: &Engine,
    condition: impl Fn(&[EventKind]) -> bool,
) -> Vec<EventKind> {
    let wait = async {
        loop {
            let kinds = event_kinds(engine).await;
            if condition(&kinds) {
                return kinds;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    };
    tokio::time::timeout(Duration::from_secs(15), wait)
        .await
        .expect("événements attendus jamais émis")
}

/// Nombre de passes terminées avec succès parmi ces événements
pub fn finished_passes(kinds: &[EventKind]) -> usize {
    let finished = |kind: &&EventKind| matches!(kind, EventKind::SyncFinished { .. });
    kinds.iter().filter(finished).count()
}

/// État final d'une opération (attendu au plus 15 s)
pub async fn operation_outcome(engine: &Engine, info: &OperationInfo) -> OperationStatus {
    let wait = async {
        loop {
            let request = Request::Operation(mira_core::api::OperationRequest::GetOperation {
                operation: info.id.clone(),
            });
            let Response::Operation(now) = engine.execute(request).await.unwrap() else {
                panic!("réponse inattendue");
            };
            if !matches!(now.status, OperationStatus::Running { .. }) {
                return now.status;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    };
    tokio::time::timeout(Duration::from_secs(15), wait)
        .await
        .expect("opération jamais terminée")
}
