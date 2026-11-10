//! Jeton à présenter au service : la clé de la connexion, ou le jeton d'accès d'une session renouvelée au besoin.

use mira_vocabulary::error::codes::{authentication, incompatible_state};
use mira_vocabulary::{
    AiConnection, AiError, AiProviderDescriptor, AiSecretRef, AiSessionLogin, ErrorCategory,
    MiraError, SuggestedAction,
};

use super::stored::read_session;
use crate::ai::access::key_for;
use crate::api::Engine;
use crate::auth::session::{AiSession, SessionRefreshError, refresh_session};
use crate::ports::SecretValue;

const RENEW_MARGIN_SECS: i64 = 120;

/// Rend la clé ; pour une connexion à session, renouvelle d'abord si l'échéance approche et range la nouvelle session avant tout usage
pub(crate) async fn credential_for(
    engine: &Engine,
    connection: &AiConnection,
    descriptor: &AiProviderDescriptor,
    base_url: &str,
) -> Result<SecretValue, MiraError> {
    let Some(login) = &descriptor.session_login else {
        return key_for(engine, connection, base_url);
    };
    let lock = engine.ai.session_locks.of(&connection.id);
    let _renewing = lock.lock().await;
    let session = read_session(engine, &connection.id).ok_or_else(missing)?;
    let now = engine.config.clock.now().as_millis() / 1000;
    if session.expires_at - now > RENEW_MARGIN_SECS {
        return Ok(SecretValue::new(session.access_token));
    }
    renew(engine, connection, login, &session, now).await
}

/// Rafraîchit, range la session de remplacement, puis rend son jeton d'accès
async fn renew(
    engine: &Engine,
    connection: &AiConnection,
    login: &AiSessionLogin,
    session: &AiSession,
    now: i64,
) -> Result<SecretValue, MiraError> {
    let fresh = refresh_session(login, session, now)
        .await
        .map_err(refresh_failure)?;
    let location = AiSecretRef {
        profile: engine.config.profile.clone(),
        connection: connection.id.clone(),
    };
    engine.config.secrets.put(&location, &fresh.to_secret())?;
    Ok(SecretValue::new(fresh.access_token))
}

/// Erreur d'un rafraîchissement : session perdue (reconnexion) ou service momentanément indisponible
fn refresh_failure(failure: SessionRefreshError) -> MiraError {
    match failure {
        SessionRefreshError::Permanent => MiraError::new(
            ErrorCategory::Authentication,
            authentication::AI_SESSION_EXPIRED,
            "la connexion à l'abonnement a expiré ou a été retirée, reconnexion nécessaire",
        )
        .with_action(SuggestedAction::Reauthenticate),
        SessionRefreshError::Transient => AiError::Unavailable.into(),
    }
}

/// Erreur d'une connexion à session sans session lisible
fn missing() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::AI_KEY_MISSING,
        "aucune session pour cette connexion d'IA, reconnexion nécessaire",
    )
}
