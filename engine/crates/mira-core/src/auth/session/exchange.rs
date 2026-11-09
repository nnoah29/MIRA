//! Échange du code d'autorisation contre une session (formulaire, sans secret de client).

use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{AiSessionLogin, ErrorCategory, MiraError};

use super::bundle::AiSession;
use super::grant::{Grant, expiry, has_scope, parse_grant};
use super::id_token::validate_id_token;
use crate::auth::flow::AuthorizationCode;
use crate::auth::tokens::{map_token_failure, post_form};
use crate::ports::SecretValue;

/// Ce qu'il faut pour échanger un code : client émis, redirection, vérificateur PKCE, nonce, hôte et heure
#[derive(Debug)]
pub struct SessionExchange<'a> {
    pub client_id: &'a str,
    pub redirect_uri: &'a str,
    pub verifier: &'a SecretValue,
    pub nonce: &'a str,
    pub host_id: &'a str,
    pub now: i64,
}

/// Échange le code, contrôle la portée exigée et le jeton d'identité, rend la session à ranger
pub async fn exchange_session_code(
    login: &AiSessionLogin,
    code: &AuthorizationCode,
    exchange: &SessionExchange<'_>,
) -> Result<AiSession, MiraError> {
    let pairs = [
        ("grant_type", "authorization_code"),
        ("client_id", exchange.client_id),
        ("code", code.expose()),
        ("code_verifier", exchange.verifier.expose()),
        ("redirect_uri", exchange.redirect_uri),
        ("resource", login.resource.as_str()),
    ];
    let (status, body) = post_form(&login.token_url, &pairs).await?;
    if !(200..300).contains(&status) {
        return Err(map_token_failure(status, &body));
    }
    build_session(login, &parse_grant(&body)?, exchange)
}

/// Contrôle portée, jeton de rafraîchissement et identité, puis compose la session
fn build_session(
    login: &AiSessionLogin,
    grant: &Grant,
    exchange: &SessionExchange<'_>,
) -> Result<AiSession, MiraError> {
    if !has_scope(grant.scope.as_deref(), &login.required_scope) {
        return Err(missing_scope());
    }
    let refresh_token = grant.refresh_token.clone().ok_or_else(missing_scope)?;
    let email = identity(login, &grant.id_token, exchange)?;
    Ok(AiSession {
        client_id: exchange.client_id.to_string(),
        host_id: exchange.host_id.to_string(),
        access_token: grant.access_token.clone(),
        refresh_token,
        expires_at: expiry(grant, exchange.now),
        email,
    })
}

/// Contrôle le jeton d'identité ; l'émetteur est l'origine de l'adresse d'autorisation
fn identity(
    login: &AiSessionLogin,
    id_token: &Option<String>,
    exchange: &SessionExchange<'_>,
) -> Result<Option<String>, MiraError> {
    let Some(token) = id_token else {
        return Err(missing_scope());
    };
    let issuer = origin_of(&login.authorize_url);
    validate_id_token(
        token,
        &issuer,
        exchange.client_id,
        exchange.nonce,
        exchange.now,
    )
}

/// Origine (schéma, hôte, port) d'une adresse, vide si elle est illisible
fn origin_of(url: &str) -> String {
    reqwest::Url::parse(url).map_or_else(
        |_| String::new(),
        |parsed| parsed.origin().ascii_serialization(),
    )
}

/// Erreur d'une autorisation qui n'accorde pas l'accès demandé
fn missing_scope() -> MiraError {
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::AI_SESSION_INCOMPLETE,
        "autorisation incomplète : accès au forfait non accordé",
    )
}
