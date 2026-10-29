//! Vérifications locales d'une génération et plan d'exécution, avant tout accès au réseau.

use std::fmt;

use mira_vocabulary::error::codes::{incompatible_state, not_found};
use mira_vocabulary::{
    AiConnection, AiConnectionId, AiError, AiProviderDescriptor, ChatRequest, ErrorCategory,
    MiraError,
};

use super::truncate::{MAX_INPUT_CHARS, truncate_request};
use crate::ai::access::{base_url_of, find_provider, is_loopback_url, key_for};
use crate::api::{ChatStart, Engine};
use crate::ports::SecretValue;

/// Tout ce que la tâche de génération doit savoir ; la requête est déjà tronquée.
pub(crate) struct ChatPlan {
    pub(crate) descriptor: AiProviderDescriptor,
    pub(crate) base_url: String,
    pub(crate) key: SecretValue,
    pub(crate) model: String,
    pub(crate) request: ChatRequest,
}

/// Vérifie connexion, fournisseur, consentement, clé puis modèle, dans cet ordre, et rend le plan.
pub(crate) fn prepare(engine: &Engine, start: ChatStart) -> Result<ChatPlan, MiraError> {
    let connection = load_connection(engine, start.connection.as_ref())?;
    let descriptor = find_provider(engine, &connection)?.descriptor();
    let base_url = base_url_of(&connection, &descriptor);
    check_consent(&connection, &base_url)?;
    let key = key_for(engine, &connection, &base_url)?;
    let model = resolve_model(&start.request, &connection)?;
    Ok(ChatPlan {
        descriptor,
        base_url,
        key,
        model,
        request: truncate_request(start.request, MAX_INPUT_CHARS),
    })
}

/// Relit la connexion désignée, ou la connexion par défaut ; absente, `not_found.ai_connection`.
fn load_connection(
    engine: &Engine,
    id: Option<&AiConnectionId>,
) -> Result<AiConnection, MiraError> {
    let found = engine.database.read(|tx| match id {
        Some(id) => tx.get_ai_connection(id),
        None => Ok(tx.list_ai_connections()?.into_iter().find(|c| c.is_default)),
    })?;
    found.ok_or_else(|| {
        MiraError::new(
            ErrorCategory::NotFound,
            not_found::AI_CONNECTION,
            "connexion d'IA inconnue",
        )
    })
}

/// Refuse une destination distante tant que l'utilisateur n'a pas consenti ; la boucle locale est exemptée.
fn check_consent(connection: &AiConnection, base_url: &str) -> Result<(), MiraError> {
    if connection.consent_at.is_none() && !is_loopback_url(base_url)? {
        return Err(AiError::ConsentRequired.into());
    }
    Ok(())
}

/// Prend le modèle de la requête, sinon celui de la connexion ; aucun, `incompatible_state.ai_model_missing`.
fn resolve_model(request: &ChatRequest, connection: &AiConnection) -> Result<String, MiraError> {
    let named = |model: &Option<String>| model.clone().filter(|name| !name.trim().is_empty());
    named(&request.model)
        .or_else(|| named(&connection.default_model))
        .ok_or_else(|| {
            MiraError::new(
                ErrorCategory::IncompatibleState,
                incompatible_state::AI_MODEL_MISSING,
                "aucun modèle d'IA choisi pour cette connexion",
            )
        })
}

impl fmt::Debug for ChatPlan {
    /// Montre le fournisseur et le modèle, jamais la clé, l'adresse ni le texte de la requête.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChatPlan")
            .field("provider", &self.descriptor.id.0)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}
