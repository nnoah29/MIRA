//! Fin d'une connexion par navigateur : le code reçu est échangé contre une clé d'API.

use mira_vocabulary::MiraError;

use super::callback::AuthorizationCode;
use super::pending::PendingFlow;
use super::phase::FlowPhase;
use super::redirect::wrong_phase;
use crate::auth::tokens::exchange_key_code;
use crate::ports::SecretValue;

/// Échange le code reçu ; le flux passe à `TokensObtained` ou `Failed`
pub async fn complete_key_flow(
    flow: &mut PendingFlow,
    code: &AuthorizationCode,
) -> Result<SecretValue, MiraError> {
    if flow.phase != FlowPhase::CodeReceived {
        return Err(wrong_phase());
    }
    let result = exchange_key_code(&flow.token_url, code, &flow.verifier).await;
    flow.phase = match &result {
        Ok(_) => FlowPhase::TokensObtained,
        Err(_) => FlowPhase::Failed,
    };
    result
}
