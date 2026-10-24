//! Remplacement des réglages du moteur : normalisés, enregistrés, appliqués tout de suite, puis annoncés.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{
    EngineSettings, ErrorCategory, EventKind, MAX_POLL_SECS, MIN_POLL_SECS, MiraError,
};

use crate::api::contract::{Response, SettingsResponse};
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Erreur de paramètre invalide
fn refused(text: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        text.to_string(),
    )
}

/// Refuse un relevé hors bornes ou des heures calmes hors d'une journée
fn validate(settings: &EngineSettings) -> Result<(), MiraError> {
    let poll_ok = settings
        .poll_secs
        .is_none_or(|secs| (MIN_POLL_SECS..=MAX_POLL_SECS).contains(&secs));
    if !poll_ok {
        return Err(refused("poll_secs hors des bornes acceptées"));
    }
    match settings.quiet_hours {
        Some(hours) if hours.from > 23 || hours.to > 23 => Err(refused(
            "quiet_hours doit être en heures pleines, de 0 à 23",
        )),
        _ => Ok(()),
    }
}

/// Réglages sous forme canonique : comptes en sourdine triés et sans doublon
fn normalized(settings: &EngineSettings) -> EngineSettings {
    let mut canonical = settings.clone();
    canonical.muted_accounts.sort();
    canonical.muted_accounts.dedup();
    canonical
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
    let settings = normalized(settings);
    let before = store(engine, &settings)?;
    let keys = before.changed_keys(&settings);
    if !keys.is_empty() {
        engine.sync.apply_settings(&settings);
        engine.events.emit(EventKind::SettingsChanged { keys })?;
    }
    Ok(Response::Settings(SettingsResponse::Settings(settings)))
}
