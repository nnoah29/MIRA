//! Fin d'une connexion par navigateur : redirection attendue, code échangé, clé rangée au trousseau.

use std::time::Duration;

use mira_vocabulary::MiraError;
use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{AiConnectionId, ErrorCategory};
use tokio::sync::Notify;

use super::connection_view::{key_location, read_connection, single};
use super::session_login_complete::{SessionCompletion, complete_session_login};
use crate::ai::access::find_provider;
use crate::ai::key_login::KeyLoginKind;
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::auth::{PendingFlow, complete_key_flow, wait_for_redirect};
use crate::ports::SecretValue;

const REDIRECT_PATIENCE: Duration = Duration::from_secs(300);

/// Attend la redirection, échange le code et range la clé ou la session ; la connexion rendue porte `has_key`
pub(crate) async fn handle_complete_ai_key_login(
    engine: &Engine,
    flow: &str,
) -> Result<Response, MiraError> {
    let (mut pending, cancel, id, kind) = engine.ai.key_logins.begin_completion(flow)?;
    let result = match &kind {
        KeyLoginKind::Key => complete_key(engine, &mut pending, &cancel, &id).await,
        KeyLoginKind::Session { host_id } => {
            complete_session(engine, &mut pending, &cancel, &id, host_id).await
        }
    };
    engine.ai.key_logins.remove(flow);
    result?;
    Ok(single(read_connection(engine, &id)?))
}

/// Termine un flux qui rend une clé d'API
async fn complete_key(
    engine: &Engine,
    pending: &mut PendingFlow,
    cancel: &Notify,
    id: &AiConnectionId,
) -> Result<(), MiraError> {
    let key = obtain_key(pending, cancel).await?;
    store_key(engine, id, &key)
}

/// Termine un flux qui rend une session de jetons
async fn complete_session(
    engine: &Engine,
    pending: &mut PendingFlow,
    cancel: &Notify,
    id: &AiConnectionId,
    host_id: &str,
) -> Result<(), MiraError> {
    let connection = read_connection(engine, id)?;
    let descriptor = find_provider(engine, &connection)?.descriptor();
    let login = descriptor.session_login.ok_or_else(cancelled)?;
    let wait = SessionCompletion {
        login: &login,
        id,
        host_id,
        cancel,
    };
    complete_session_login(engine, pending, &wait).await
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
pub(super) fn cancelled() -> MiraError {
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::OAUTH_CANCELLED,
        "connexion annulée",
    )
}
