//! Appels du domaine « settings » par la façade et lecture de leurs réponses.

use mira_core::api::{Engine, Request, Response, SettingsRequest, SettingsResponse};
use mira_vocabulary::{EngineSettings, EventKind};

use crate::support_api::queries::event_kinds;

/// Exécute une requête du domaine et rend les réglages en vigueur, ou le code d'erreur
pub async fn call(engine: &Engine, request: SettingsRequest) -> Result<EngineSettings, String> {
    match engine.execute(Request::Settings(request)).await {
        Ok(Response::Settings(SettingsResponse::Settings(settings))) => Ok(settings),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Réglages en vigueur
pub async fn current(engine: &Engine) -> EngineSettings {
    call(engine, SettingsRequest::GetSettings).await.unwrap()
}

/// Remplace les réglages par un relevé périodique donné
pub async fn set_poll(engine: &Engine, poll_secs: Option<u32>) -> Result<EngineSettings, String> {
    let settings = EngineSettings { poll_secs };
    call(engine, SettingsRequest::SetSettings { settings }).await
}

/// Clés annoncées par chaque événement « réglages changés », dans l'ordre
pub async fn announced_keys(engine: &Engine) -> Vec<Vec<String>> {
    let kinds = event_kinds(engine).await;
    kinds
        .into_iter()
        .filter_map(|kind| match kind {
            EventKind::SettingsChanged { keys } => Some(keys),
            _ => None,
        })
        .collect()
}
