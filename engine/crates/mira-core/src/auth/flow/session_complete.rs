//! Fin d'une connexion qui rend une session : le code reçu est échangé contre des jetons, le client émis est retenu.

use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{AiSessionLogin, ErrorCategory, MiraError};

use super::callback::AuthorizationCode;
use super::pending::PendingFlow;
use super::phase::FlowPhase;
use super::redirect::wrong_phase;
use crate::auth::session::{AiSession, SessionExchange, exchange_session_code};

/// Échange le code ; le flux passe à `TokensObtained` ou `Failed`
pub async fn complete_session_flow(
    flow: &mut PendingFlow,
    login: &AiSessionLogin,
    code: &AuthorizationCode,
    known: Option<&AiSession>,
    host_id: &str,
    now: i64,
) -> Result<AiSession, MiraError> {
    if flow.phase != FlowPhase::CodeReceived {
        return Err(wrong_phase());
    }
    let result = exchange(flow, login, code, known, host_id, now).await;
    flow.phase = match &result {
        Ok(_) => FlowPhase::TokensObtained,
        Err(_) => FlowPhase::Failed,
    };
    result
}

/// Retient le client émis puis échange le code
async fn exchange(
    flow: &PendingFlow,
    login: &AiSessionLogin,
    code: &AuthorizationCode,
    known: Option<&AiSession>,
    host_id: &str,
    now: i64,
) -> Result<AiSession, MiraError> {
    let client_id = issued_client(known, code)?;
    let nonce = flow.nonce.as_deref().unwrap_or_default();
    let exchange = SessionExchange {
        client_id: &client_id,
        redirect_uri: &flow.redirect_uri,
        verifier: &flow.verifier,
        nonce,
        host_id,
        now,
    };
    exchange_session_code(login, code, &exchange).await
}

/// Client à utiliser : celui déjà connu, sinon celui que la redirection d'inscription a émis ; un écart est refusé
fn issued_client(known: Option<&AiSession>, code: &AuthorizationCode) -> Result<String, MiraError> {
    let issued = code.issued_client_id();
    match (known.map(|session| session.client_id.as_str()), issued) {
        (Some(old), Some(new)) if old != new => Err(incomplete()),
        (Some(old), _) => Ok(old.to_string()),
        (None, Some(new)) if !new.is_empty() => Ok(new.to_string()),
        _ => Err(incomplete()),
    }
}

/// Erreur d'une inscription qui ne rend pas d'identifiant de client
fn incomplete() -> MiraError {
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::REJECTED,
        "inscription incomplète : aucun identifiant de client reçu",
    )
}
