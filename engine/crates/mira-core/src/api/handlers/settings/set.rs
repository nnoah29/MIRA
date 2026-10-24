//! Remplacement des réglages du moteur : enregistrés, appliqués tout de suite, puis annoncés.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{
    EngineSettings, ErrorCategory, EventKind, MAX_POLL_SECS, MIN_POLL_SECS, MiraError,
};

use crate::api::contract::{Response, SettingsResponse};
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Refuse un relevé périodique hors des bornes acceptées
fn validate(settings: &EngineSettings) -> Result<(), MiraError> {
    match settings.poll_secs {
        Some(secs) if !(MIN_POLL_SECS..=MAX_POLL_SECS).contains(&secs) => Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            format!("poll_secs doit être entre {MIN_POLL_SECS} et {MAX_POLL_SECS}"),
        )),
        _ => Ok(()),
    }
}

/// Enregistre les réglages et rend ceux d'avant
fn store(engine: &Engine, settings: &EngineSettings) -> Result<EngineSettings, MiraError> {
    engine.database.write(|tx| {
        let before = tx.get_engine_settings()?;
        tx.put_engine_settings(settings)?;
        Ok(before)
    })
}

/// Remplace les réglages ; applique aux tâches en cours puis annonce seulement si quelque chose change
pub(crate) fn handle_set_settings(
    engine: &Engine,
    settings: &EngineSettings,
) -> Result<Response, MiraError> {
    validate(settings)?;
    let before = store(engine, settings)?;
    let keys = before.changed_keys(settings);
    if !keys.is_empty() {
        engine.sync.apply_settings(settings);
        engine.events.emit(EventKind::SettingsChanged { keys })?;
    }
    Ok(Response::Settings(SettingsResponse::Settings(*settings)))
}
