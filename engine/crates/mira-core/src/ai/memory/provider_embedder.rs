//! Embedder du mode fournisseur : adresse et clé de la connexion choisie, consentement et type « mémoire » exigés.

use mira_vocabulary::error::codes::incompatible_state::AI_CONTENT_NOT_ALLOWED;
use mira_vocabulary::{
    AiConnection, AiConnectionId, AiError, ContentKind, ErrorCategory, MiraError,
};

use super::http_embedder::HttpEmbedder;
use crate::ai::access::{base_url_of, find_provider, is_loopback_url};
use crate::ai::session::credential_for;
use crate::ai::transport::TransportLimits;
use crate::api::Engine;

/// Refuse l'envoi tant que la connexion n'a pas de consentement et n'autorise pas le type « mémoire » ; la boucle locale est exemptée.
fn check_sending(connection: &AiConnection, base_url: &str) -> Result<(), MiraError> {
    if is_loopback_url(base_url)? {
        return Ok(());
    }
    if connection.consent_at.is_none() {
        return Err(AiError::ConsentRequired.into());
    }
    if !connection.allowed_kinds.contains(&ContentKind::Memory) {
        return Err(MiraError::new(
            ErrorCategory::IncompatibleState,
            AI_CONTENT_NOT_ALLOWED,
            "ce service d'IA n'est pas autorisé à recevoir les souvenirs",
        ));
    }
    Ok(())
}

/// Embedder branché sur la connexion ; le modèle est qualifié `<fournisseur>:<modèle>` pour ne jamais mélanger deux services
pub(crate) async fn provider_embedder(
    engine: &Engine,
    id: &AiConnectionId,
    model: &str,
) -> Result<HttpEmbedder, MiraError> {
    let connection = engine.database.read(|tx| tx.require_ai_connection(id))?;
    let descriptor = find_provider(engine, &connection)?.descriptor();
    let base_url = base_url_of(&connection, &descriptor);
    check_sending(&connection, &base_url)?;
    let key = credential_for(engine, &connection, &descriptor, &base_url).await?;
    let qualified = format!("{}:{model}", connection.provider.0);
    let embedder = HttpEmbedder::new(&base_url, key, qualified, TransportLimits::default());
    Ok(embedder.with_path("/embeddings"))
}
