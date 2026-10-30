//! Flux d'une génération d'IA vers le client : une ligne JSON par delta, annulation si le flux Dart se ferme.

use mira_assembly::api::{ChatStart, ChatStream, OperationRequest, Request};
use mira_assembly::mira_vocabulary::error::codes::internal;
use mira_assembly::mira_vocabulary::{
    AiError, ChatDelta, ErrorCategory, MiraError, OperationId, StopReason,
};
use serde_json::json;

use crate::session::current;

/// Ligne d'erreur : le code stable et le message du moteur, déjà sans secret.
fn error_line(error: &MiraError) -> String {
    json!({"type": "error", "error": {"code": error.code, "message": error.message}}).to_string()
}

/// Erreur interne du pont, au message fixe (jamais le texte de la requête).
fn bridge_error(message: &str) -> MiraError {
    MiraError::new(ErrorCategory::Internal, internal::BUG, message)
}

/// Lit la demande puis lance la génération sur le moteur ouvert, avec les outils en lecture seule si la demande les permet.
async fn open_stream(request_json: &str) -> Result<ChatStream, MiraError> {
    let start: ChatStart =
        serde_json::from_str(request_json).map_err(|_| bridge_error("requête JSON illisible"))?;
    let engine = current().ok_or_else(|| bridge_error("moteur non ouvert"))?;
    engine.chat_stream_with_tools(start).await
}

/// Annule l'opération par le contrat, comme le ferait l'application ; l'échec éventuel est sans objet.
async fn cancel_chat(operation: &OperationId) {
    let Some(engine) = current() else { return };
    let operation = operation.clone();
    let request = Request::Operation(OperationRequest::CancelOperation { operation });
    let _ = engine.execute(request).await;
}

/// Ligne terminale de secours quand le flux s'arrête sans `done` alors que le client écoute encore.
pub fn closing_line(finished: bool, open: bool) -> Option<String> {
    if finished || !open {
        return None;
    }
    let code = AiError::Unavailable.code().0.to_string();
    let reason = StopReason::Failed {
        code,
        retry_after_secs: None,
    };
    serde_json::to_string(&ChatDelta::Done { reason }).ok()
}

/// Envoie `started`, puis chaque delta (JSON) à `emit`, ou une seule ligne `error` ; si `emit` rend faux, annule.
pub async fn pump_chat(request_json: &str, mut emit: impl FnMut(String) -> bool) {
    let mut stream = match open_stream(request_json).await {
        Ok(stream) => stream,
        Err(error) => {
            emit(error_line(&error));
            return;
        }
    };
    let started = json!({"type": "started", "operation": stream.operation});
    let mut open = emit(started.to_string());
    let mut finished = false;
    while open
        && !finished
        && let Some(delta) = stream.next().await
    {
        finished = matches!(delta, ChatDelta::Done { .. });
        open = serde_json::to_string(&delta).is_ok_and(&mut emit);
    }
    if let Some(line) = closing_line(finished, open) {
        emit(line);
    }
    if !open && !finished {
        cancel_chat(&stream.operation).await;
    }
}
