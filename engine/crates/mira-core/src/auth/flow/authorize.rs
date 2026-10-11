//! Déroulement d'un flux jusqu'aux jetons, interrompu proprement par une annulation.

use std::time::Duration;

use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{ErrorCategory, MiraError};
use tokio::sync::Notify;

use super::complete::complete_flow;
use super::pending::PendingFlow;
use super::redirect::wait_for_redirect;
use crate::auth::tokens::TokenSet;
use crate::ports::Clock;

/// Attend la redirection (ou l'annulation), vérifie `state`, puis échange le code contre des jetons
pub async fn authorize_until_cancelled(
    flow: &mut PendingFlow,
    cancel: &Notify,
    patience: Duration,
    clock: &dyn Clock,
) -> Result<TokenSet, MiraError> {
    let code = tokio::select! {
        received = wait_for_redirect(flow, patience) => received?,
        () = cancel.notified() => return Err(cancelled()),
    };
    complete_flow(flow, &code, clock).await
}

/// Erreur d'un flux annulé par l'utilisateur
fn cancelled() -> MiraError {
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::OAUTH_CANCELLED,
        "autorisation annulée",
    )
}
