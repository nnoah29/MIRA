//! Vérifications locales d'une génération : consentement, types de contenu, clé, modèle, résumé du contexte.

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{
    AiConnection, AiError, AiProviderDescriptor, ChatRequest, ContentKind, ErrorCategory, MiraError,
};

use super::kinds::{check_kinds, distinct_kinds};
use crate::ai::access::{base_url_of, find_provider, is_loopback_url};
use crate::ai::local::local_endpoint;
use crate::ai::session::credential_for;
use crate::api::{ChatStart, Engine};
use crate::ports::SecretValue;

/// Ce que les vérifications locales établissent sur la destination et la clé.
pub(super) struct Checked {
    pub(super) descriptor: AiProviderDescriptor,
    pub(super) base_url: String,
    pub(super) key: SecretValue,
    pub(super) model: String,
    pub(super) local: bool,
}

/// Contrôle consentement, types de contenu, clé et modèle ; rend la destination résolue.
pub(super) async fn check(
    engine: &Engine,
    connection: &AiConnection,
    start: &ChatStart,
) -> Result<Checked, MiraError> {
    let descriptor = find_provider(engine, connection)?.descriptor();
    if descriptor.local_runtime.is_some() {
        return check_local(engine, connection, start, descriptor).await;
    }
    let base_url = base_url_of(connection, &descriptor);
    check_consent(connection, &base_url)?;
    let local = is_loopback_url(&base_url).unwrap_or(false);
    check_kinds(connection, &start.request.context, local)?;
    let key = credential_for(engine, connection, &descriptor, &base_url).await?;
    let model = resolve_model(&start.request, connection)?;
    Ok(Checked {
        descriptor,
        base_url,
        key,
        model,
        local,
    })
}

/// Contrôle une connexion locale : pas de consentement (rien ne sort), modèle choisi, serveur lancé.
async fn check_local(
    engine: &Engine,
    connection: &AiConnection,
    start: &ChatStart,
    descriptor: AiProviderDescriptor,
) -> Result<Checked, MiraError> {
    check_kinds(connection, &start.request.context, true)?;
    let model = resolve_model(&start.request, connection)?;
    let handle = local_endpoint(engine, &descriptor, &model).await?;
    Ok(Checked {
        descriptor,
        base_url: handle.base_url,
        key: handle.api_key,
        model,
        local: true,
    })
}

/// Types de contenu distincts et nombre de parties du contexte, pour le journal.
pub(super) fn context_summary(request: &ChatRequest) -> (Vec<ContentKind>, u32) {
    let parts = u32::try_from(request.context.len()).unwrap_or(u32::MAX);
    (distinct_kinds(&request.context), parts)
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
