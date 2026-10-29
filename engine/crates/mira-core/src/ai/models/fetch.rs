//! Relevé de la liste des modèles d'une connexion, avec le cache gardé en base.

use mira_vocabulary::{AiConnection, AiError, AiModelList, AiProviderDescriptor, MiraError};

use super::access::{base_url_of, find_provider, key_for};
use super::{filter_models, sort_models};
use crate::ai::transport::{TransportLimits, get_bounded};
use crate::ai::wire::dispatch::decode_models;
use crate::api::Engine;

/// Rend la liste gardée si elle existe et qu'on ne rafraîchit pas, sinon interroge le service puis la garde.
pub async fn fetch_models(
    engine: &Engine,
    connection: &AiConnection,
    refresh: bool,
) -> Result<AiModelList, MiraError> {
    if let (false, Some(cached)) = (refresh, &connection.models) {
        return Ok(cached.clone());
    }
    let descriptor = find_provider(engine, connection)?.descriptor();
    let base_url = base_url_of(connection, &descriptor);
    let key = key_for(engine, connection, &base_url)?;
    let path = descriptor.models_path.clone();
    let body = get_bounded(
        &base_url,
        &descriptor,
        &key,
        &path,
        &TransportLimits::default(),
    )
    .await?;
    let list = build_list(engine, &descriptor, &body)?;
    engine
        .database
        .write(|tx| tx.save_ai_models(&connection.id, &list))?;
    Ok(list)
}

/// Décode la réponse, retire les exclusions du fournisseur, trie et date le relevé.
fn build_list(
    engine: &Engine,
    descriptor: &AiProviderDescriptor,
    body: &[u8],
) -> Result<AiModelList, AiError> {
    let decoded = decode_models(descriptor.format, body)?;
    let models = sort_models(filter_models(decoded, &descriptor.model_exclusions));
    Ok(AiModelList {
        models,
        fetched_at: engine.config.clock.now(),
    })
}
