#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Appels du domaine « system_label » par la façade et lecture de leurs réponses.

use mira_core::api::{Engine, Request, Response, SystemLabelRequest, SystemLabelResponse};
use mira_vocabulary::{EventKind, SystemLabelAssignment, SystemLabelKind as Kind, ThreadId};

use crate::support_api::queries::event_kinds;

/// Exécute une requête du domaine et rend sa réponse, ou le code d'erreur
pub async fn call(
    engine: &Engine,
    request: SystemLabelRequest,
) -> Result<SystemLabelResponse, String> {
    match engine.execute(Request::SystemLabel(request)).await {
        Ok(Response::SystemLabel(answer)) => Ok(answer),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Pose (ou retire) une sorte sur ces conversations
pub async fn set(
    engine: &Engine,
    kind: Kind,
    threads: &[ThreadId],
    on: bool,
) -> Result<(), String> {
    let request = SystemLabelRequest::SetSystemLabel {
        kind,
        threads: threads.to_vec(),
        on,
    };
    call(engine, request).await.map(|_| ())
}

/// Poses listées par le moteur
pub async fn listed(engine: &Engine) -> Vec<SystemLabelAssignment> {
    match call(engine, SystemLabelRequest::ListSystemLabelAssignments).await {
        Ok(SystemLabelResponse::Assignments(items)) => items,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Nombre d'événements `SystemLabelsChanged` émis depuis le début
pub async fn changed_count(engine: &Engine) -> usize {
    let kinds = event_kinds(engine).await;
    kinds
        .iter()
        .filter(|k| matches!(k, EventKind::SystemLabelsChanged))
        .count()
}
