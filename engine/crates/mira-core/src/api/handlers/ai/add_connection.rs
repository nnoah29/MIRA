//! Création d'une connexion d'IA : fournisseur connu, libellé et adresse contrôlés, jamais défaut d'office.

use mira_vocabulary::{AiConnection, AiConnectionId, AiProviderId, MiraError};

use super::connection_view::{read_connection, single};
use super::validate_label::validate_label;
use super::validate_url::resolve_base_url;
use crate::ai::access::find_provider_by_id;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Crée la connexion sous un identifiant du moteur, sans clé, sans consentement, hors défaut
pub(crate) fn handle_add_ai_connection(
    engine: &Engine,
    provider: &AiProviderId,
    label: &str,
    base_url: Option<&str>,
) -> Result<Response, MiraError> {
    let descriptor = find_provider_by_id(engine, provider)?.descriptor();
    let label = validate_label(label)?;
    let base_url = resolve_base_url(&descriptor, base_url)?;
    let id = AiConnectionId::from_raw(engine.config.ids.next_id());
    let now = engine.config.clock.now().as_millis();
    let connection = blank_connection(id.clone(), provider, label, base_url);
    engine
        .database
        .write(|tx| tx.insert_ai_connection(&connection, now))?;
    Ok(single(read_connection(engine, &id)?))
}

/// Connexion neuve : ni défaut, ni modèle, ni consentement, ni liste gardée
fn blank_connection(
    id: AiConnectionId,
    provider: &AiProviderId,
    label: String,
    base_url: Option<String>,
) -> AiConnection {
    AiConnection {
        id,
        provider: provider.clone(),
        label,
        base_url,
        default_model: None,
        is_default: false,
        has_key: false,
        consent_at: None,
        models: None,
    }
}
