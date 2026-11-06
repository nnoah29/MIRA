//! Lignes JSON du flux de chat qui ne viennent pas du moteur : erreur, fin de secours.

use mira_assembly::mira_vocabulary::error::codes::internal;
use mira_assembly::mira_vocabulary::{AiError, ChatDelta, ErrorCategory, MiraError, StopReason};
use serde_json::json;

/// Ligne d'erreur : le code stable et le message du moteur, déjà sans secret.
pub fn error_line(error: &MiraError) -> String {
    json!({"type": "error", "error": {"code": error.code, "message": error.message}}).to_string()
}

/// Erreur interne du pont, au message fixe (jamais le texte de la requête).
pub fn bridge_error(message: &str) -> MiraError {
    MiraError::new(ErrorCategory::Internal, internal::BUG, message)
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
