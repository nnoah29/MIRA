//! Mise à jour d'une connexion d'IA : libellé, adresse, modèle ; une autre adresse efface l'ancienne destination.

use mira_vocabulary::{AiConnectionId, MaskingSettings, MiraError};

use super::connection_view::{key_location, read_connection, single};
use super::default_masking::default_masking;
use super::validate_label::validate_label;
use super::validate_url::resolve_base_url;
use crate::ai::access::find_provider;
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::storage::Tx;

/// Remplace libellé, adresse et modèle par défaut ; consentement, types, clé et modèles gardés ne valent que pour l'adresse d'origine
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
    let moved = base_url != current.base_url;
    let masking = default_masking(&descriptor.base_url, base_url.as_deref());
    if moved {
        engine.config.secrets.delete(&key_location(engine, id))?;
    }
    engine.database.write(|tx| {
        tx.update_ai_connection(id, &label, base_url.as_deref(), model, now)?;
        if moved {
            forget_destination(tx, id, &masking)?;
        }
        Ok(())
    })?;
    Ok(single(read_connection(engine, id)?))
}

/// Oublie ce qui appartenait à l'ancienne destination (consentement, types, modèles) et reprend le masquage de la nouvelle
fn forget_destination(
    tx: &Tx<'_>,
    id: &AiConnectionId,
    masking: &MaskingSettings,
) -> Result<(), MiraError> {
    tx.clear_ai_consent(id)?;
    tx.set_ai_allowed_kinds(id, &[])?;
    tx.set_ai_masking(id, masking)?;
    tx.clear_ai_models(id)
}
