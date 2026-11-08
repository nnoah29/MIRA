//! Fin d'une connexion par navigateur : redirection attendue, code échangé, clé rangée au trousseau.

use std::time::Duration;

use mira_vocabulary::MiraError;
use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{AiConnectionId, ErrorCategory};
use tokio::sync::Notify;

use super::connection_view::{key_location, read_connection, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::auth::{PendingFlow, complete_key_flow, wait_for_redirect};
use crate::ports::SecretValue;

const REDIRECT_PATIENCE: Duration = Duration::from_secs(300);

/// Attend la redirection, échange le code et range la clé ; la connexion rendue porte `has_key`
pub(crate) async fn handle_complete_ai_key_login(
    engine: &Engine,
    flow: &str,
) -> Result<Response, MiraError> {
    let (mut pending, cancel, id) = engine.ai_key_logins.begin_completion(flow)?;
    let result = obtain_key(&mut pending, &cancel).await;
    engine.ai_key_logins.remove(flow);
    let key = result?;
    store_key(engine, &id, &key)?;
    Ok(single(read_connection(engine, &id)?))
}

/// Attend la redirection (ou l'annulation) puis échange le code contre la clé
async fn obtain_key(pending: &mut PendingFlow, cancel: &Notify) -> Result<SecretValue, MiraError> {
    let code = tokio::select! {
        received = wait_for_redirect(pending, REDIRECT_PATIENCE) => received?,
        () = cancel.notified() => return Err(cancelled()),
    };
    complete_key_flow(pending, &code).await
}

/// Range la clé reçue ; la clé précédente n'est remplacée qu'à ce moment
fn store_key(engine: &Engine, id: &AiConnectionId, key: &SecretValue) -> Result<(), MiraError> {
    engine.config.secrets.put(&key_location(engine, id), key)
}

/// Erreur d'une connexion annulée par l'utilisateur
fn cancelled() -> MiraError {
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::OAUTH_CANCELLED,
        "connexion annulée",
    )
}
