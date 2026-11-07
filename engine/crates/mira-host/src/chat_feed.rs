//! Flux d'une génération d'IA vers le client : une ligne JSON par delta, annulation si le client se détache.

use std::future::Future;

use mira_assembly::api::{ChatStart, ChatStream, OperationRequest, Request};
use mira_assembly::mira_vocabulary::{ChatDelta, MiraError, OperationId};
use serde_json::json;

use crate::Host;
pub use crate::chat_lines::closing_line;
use crate::chat_lines::{bridge_error, error_line};

/// Lit la demande puis lance la génération sur le moteur ouvert, avec les outils en lecture seule si la demande les permet.
async fn open_stream(host: &Host, request_json: &str) -> Result<ChatStream, MiraError> {
    let start: ChatStart =
        serde_json::from_str(request_json).map_err(|_| bridge_error("requête JSON illisible"))?;
    let engine = host
        .engine()
        .ok_or_else(|| bridge_error("moteur non ouvert"))?;
    engine.chat_stream_with_tools(start).await
}

/// Annule l'opération par le contrat, comme le ferait l'application ; l'échec éventuel est sans objet.
async fn cancel_chat(host: &Host, operation: &OperationId) {
    let Some(engine) = host.engine() else { return };
    let operation = operation.clone();
    let request = Request::Operation(OperationRequest::CancelOperation { operation });
    let _ = engine.execute(request).await;
}

/// Envoie un delta au client ; faux si le delta est illisible ou si le client ne suit plus.
async fn forward<F, Fut>(delta: &ChatDelta, emit: &mut F) -> bool
where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = bool>,
{
    match serde_json::to_string(delta) {
        Ok(line) => emit(line).await,
        Err(_) => false,
    }
}

/// Envoie `started`, puis chaque delta (JSON) à `emit`, ou une seule ligne `error` ; si `emit` rend faux, annule.
pub async fn pump_chat<F, Fut>(host: &Host, request_json: &str, mut emit: F)
where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = bool>,
{
    let mut stream = match open_stream(host, request_json).await {
        Ok(stream) => stream,
        Err(error) => {
            emit(error_line(&error)).await;
            return;
        }
    };
    let started = json!({"type": "started", "operation": stream.operation});
    let mut open = emit(started.to_string()).await;
    let mut finished = false;
    while open
        && !finished
        && let Some(delta) = stream.next().await
    {
        finished = matches!(delta, ChatDelta::Done { .. });
        open = forward(&delta, &mut emit).await;
    }
    if let Some(line) = closing_line(finished, open) {
        emit(line).await;
    }
    if !open && !finished {
        cancel_chat(host, &stream.operation).await;
    }
}
