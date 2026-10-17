//! Fin du flux : échange du code reçu contre des jetons et passage à l'étape finale.

use mira_vocabulary::MiraError;

use super::callback::AuthorizationCode;
use super::pending::PendingFlow;
use super::phase::FlowPhase;
use super::redirect::wrong_phase;
use crate::auth::tokens::{TokenSet, exchange_code_with_secret};
use crate::ports::Clock;

/// Échange le code reçu ; le flux passe à `TokensObtained` ou `Failed`
pub async fn complete_flow(
    flow: &mut PendingFlow,
    code: &AuthorizationCode,
    clock: &dyn Clock,
) -> Result<TokenSet, MiraError> {
    if flow.phase != FlowPhase::CodeReceived {
        return Err(wrong_phase());
    }
    let result = exchange_code_with_secret(
        &flow.token_url,
        (&flow.client_id, flow.client_secret.as_ref()),
        code,
        &flow.verifier,
        &flow.redirect_uri,
        clock,
    )
    .await;
    flow.phase = match &result {
        Ok(_) => FlowPhase::TokensObtained,
        Err(_) => FlowPhase::Failed,
    };
    result
}
