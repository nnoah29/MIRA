//! Fin d'une connexion qui rend une session : redirection attendue, code échangé, session rangée au trousseau.

use std::time::Duration;

use mira_vocabulary::{AiConnectionId, AiSessionLogin, MiraError};
use tokio::sync::Notify;

use super::connection_view::key_location;
use crate::ai::session::read_session;
use crate::api::engine::Engine;
use crate::auth::{PendingFlow, complete_session_flow, wait_for_redirect};

const REDIRECT_PATIENCE: Duration = Duration::from_secs(300);

/// Ce qu'il faut pour terminer le flux de session
pub(super) struct SessionCompletion<'a> {
    pub(super) login: &'a AiSessionLogin,
    pub(super) id: &'a AiConnectionId,
    pub(super) host_id: &'a str,
    pub(super) cancel: &'a Notify,
}

/// Attend la redirection, échange le code et range la session ; la précédente n'est remplacée qu'à ce moment
pub(super) async fn complete_session_login(
    engine: &Engine,
    pending: &mut PendingFlow,
    wait: &SessionCompletion<'_>,
) -> Result<(), MiraError> {
    let code = tokio::select! {
        received = wait_for_redirect(pending, REDIRECT_PATIENCE) => received?,
        () = wait.cancel.notified() => return Err(super::key_login_complete::cancelled()),
    };
    let known = read_session(engine, wait.id);
    let now = engine.config.clock.now().as_millis() / 1000;
    let session = complete_session_flow(
        pending,
        wait.login,
        &code,
        known.as_ref(),
        wait.host_id,
        now,
    )
    .await?;
    engine
        .config
        .secrets
        .put(&key_location(engine, wait.id), &session.to_secret())
}
