//! Erreurs de l'attente : délai dépassé, opération échouée, action en échec ou incertaine.

use mira_assembly::mira_vocabulary::error::codes::{network, server};
use mira_assembly::mira_vocabulary::{ActionState, ErrorCategory, MiraError, PendingAction};
use serde_json::Value;

/// Délai dépassé : l'action ou l'opération continue de son côté, rien n'est perdu.
pub fn timeout_error(seconds: u64) -> MiraError {
    MiraError::new(
        ErrorCategory::Network,
        network::TIMEOUT,
        format!("attente dépassée après {seconds} s ; l'action reste en file"),
    )
}

/// Erreur du contrat reconstruite à partir du code `<catégorie>.<sujet>` d'une opération échouée.
pub fn error_from_code(code: &str) -> MiraError {
    let prefix = code.split('.').next().unwrap_or_default();
    let category = serde_json::from_value::<ErrorCategory>(Value::String(prefix.to_string()))
        .unwrap_or(ErrorCategory::Internal);
    MiraError {
        category,
        code: code.to_string(),
        message: format!("l'opération a échoué ({code})"),
        retryable: category == ErrorCategory::Network,
        suggested_action: None,
        context: Box::default(),
    }
}

/// Erreur d'une action terminée en échec ou en résultat incertain.
pub fn broken_action_error(action: &PendingAction) -> MiraError {
    let detail = action.last_error.clone().unwrap_or_default();
    match action.state {
        ActionState::Uncertain => MiraError::new(
            ErrorCategory::Network,
            network::CONNECTION_LOST,
            format!("action {} : résultat incertain ({detail})", action.id),
        ),
        _ => MiraError::new(
            ErrorCategory::Server,
            server::REJECTED,
            format!("action {} en échec ({detail})", action.id),
        ),
    }
}
