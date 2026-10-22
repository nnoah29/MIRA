//! Appels du domaine « local_label » par la façade et lecture de leurs réponses.

use mira_core::api::{Engine, LocalLabelRequest, LocalLabelResponse, Request, Response};
use mira_vocabulary::{EventKind, IdempotencyKey, LocalLabel, MiraError, ThreadId};

use crate::support_api::queries::event_kinds;

/// Exécute une requête du domaine et rend la réponse du domaine, ou le code d'erreur
pub async fn call(
    engine: &Engine,
    request: LocalLabelRequest,
) -> Result<LocalLabelResponse, String> {
    match engine.execute(Request::LocalLabel(request)).await {
        Ok(Response::LocalLabel(answer)) => Ok(answer),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Clé d'idempotence de test
pub fn key(raw: &str) -> IdempotencyKey {
    IdempotencyKey(raw.to_string())
}

/// Crée un libellé sous cette clé et le rend, ou le code d'erreur
pub async fn create(engine: &Engine, name: &str, key_text: &str) -> Result<LocalLabel, String> {
    let request = LocalLabelRequest::CreateLocalLabel {
        name: name.to_string(),
        color: None,
        idempotency_key: key(key_text),
    };
    match call(engine, request).await? {
        LocalLabelResponse::Label(label) => Ok(label),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Libellés listés par le moteur
pub async fn labels(engine: &Engine) -> Vec<LocalLabel> {
    match call(engine, LocalLabelRequest::ListLocalLabels)
        .await
        .unwrap()
    {
        LocalLabelResponse::Labels(found) => found,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Pose ou retire un libellé sur des fils sous cette clé
pub async fn set(
    engine: &Engine,
    label: &LocalLabel,
    threads: &[ThreadId],
    on: bool,
    key_text: &str,
) -> Result<(), String> {
    let request = LocalLabelRequest::SetLocalLabel {
        label: label.id.clone(),
        threads: threads.to_vec(),
        on,
        idempotency_key: key(key_text),
    };
    call(engine, request).await.map(|_| ())
}

/// Nombre d'événements `LocalLabelsChanged` émis depuis le début
pub async fn changed_count(engine: &Engine) -> usize {
    let kinds = event_kinds(engine).await;
    kinds
        .iter()
        .filter(|kind| matches!(kind, EventKind::LocalLabelsChanged))
        .count()
}
