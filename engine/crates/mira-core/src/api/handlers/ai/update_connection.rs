//! Mise à jour d'une connexion d'IA : libellé, adresse, modèle ; une autre adresse annule le consentement.

use mira_vocabulary::{AiConnectionId, MiraError};

use super::connection_view::{read_connection, single};
use super::validate_label::validate_label;
use super::validate_url::resolve_base_url;
use crate::ai::access::find_provider;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Remplace libellé, adresse et modèle par défaut ; le consentement ne vaut que pour l'adresse consentie
pub(crate) fn handle_update_ai_connection(
    engine: &Engine,
    id: &AiConnectionId,
    label: &str,
    base_url: Option<String>,
    default_model: Option<String>,
) -> Result<Response, MiraError> {
    let current = read_connection(engine, id)?;
    let descriptor = find_provider(engine, &current)?.descriptor();
    let label = validate_label(label)?;
    let base_url = resolve_base_url(&descriptor, base_url.as_deref())?;
    let model = default_model
        .as_deref()
        .map(str::trim)
        .filter(|model| !model.is_empty());
    let now = engine.config.clock.now().as_millis();
    engine.database.write(|tx| {
        tx.update_ai_connection(id, &label, base_url.as_deref(), model, now)?;
        if base_url != current.base_url {
            tx.clear_ai_consent(id)?;
        }
        Ok(())
    })?;
    Ok(single(read_connection(engine, id)?))
}
