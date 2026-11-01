//! Raccourcis de l'interrupteur de l'historique des conversations d'IA.

use mira_core::api::{AiRequest, AiResponse, Engine};

use super::call;

/// Lit l'interrupteur de l'historique
pub async fn enabled(engine: &Engine) -> bool {
    match call(engine, AiRequest::GetAiHistoryEnabled).await.unwrap() {
        AiResponse::HistoryEnabled(value) => value,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Règle l'interrupteur et rend la valeur confirmée par le moteur, ou le code d'erreur
pub async fn set_enabled(engine: &Engine, value: bool) -> Result<bool, String> {
    let request = AiRequest::SetAiHistoryEnabled { enabled: value };
    match call(engine, request).await? {
        AiResponse::HistoryEnabled(confirmed) => Ok(confirmed),
        other => panic!("réponse inattendue : {other:?}"),
    }
}
