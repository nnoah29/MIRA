//! Échange du code d'autorisation et rafraîchissement silencieux des jetons (7.7, 15.2).

use mira_vocabulary::MiraError;

use super::failure::map_token_failure;
use super::http::post_form;
use super::response::parse_tokens;
use super::set::TokenSet;
use crate::auth::flow::AuthorizationCode;
use crate::ports::{Clock, SecretValue};

/// Échange le code contre des jetons en présentant le vérificateur PKCE
pub async fn exchange_code(
    token_url: &str,
    client_id: &str,
    code: &AuthorizationCode,
    verifier: &SecretValue,
    redirect_uri: &str,
    clock: &dyn Clock,
) -> Result<TokenSet, MiraError> {
    exchange_code_with_secret(
        token_url,
        (client_id, None),
        code,
        verifier,
        redirect_uri,
        clock,
    )
    .await
}

/// Même échange pour un client qui présente aussi un secret d'application
pub async fn exchange_code_with_secret(
    token_url: &str,
    client: (&str, Option<&SecretValue>),
    code: &AuthorizationCode,
    verifier: &SecretValue,
    redirect_uri: &str,
    clock: &dyn Clock,
) -> Result<TokenSet, MiraError> {
    let mut pairs = vec![
        ("grant_type", "authorization_code"),
        ("code", code.expose()),
        ("client_id", client.0),
        ("redirect_uri", redirect_uri),
        ("code_verifier", verifier.expose()),
    ];
    push_secret(&mut pairs, client.1);
    request_tokens(token_url, &pairs, clock).await
}

/// Obtient un nouveau jeton d'accès à partir du jeton de rafraîchissement
pub async fn refresh_tokens(
    token_url: &str,
    client_id: &str,
    refresh_token: &SecretValue,
    clock: &dyn Clock,
) -> Result<TokenSet, MiraError> {
    refresh_tokens_with_secret(token_url, (client_id, None), refresh_token, clock).await
}

/// Même rafraîchissement pour un client qui présente aussi un secret d'application
pub async fn refresh_tokens_with_secret(
    token_url: &str,
    client: (&str, Option<&SecretValue>),
    refresh_token: &SecretValue,
    clock: &dyn Clock,
) -> Result<TokenSet, MiraError> {
    let mut pairs = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.expose()),
        ("client_id", client.0),
    ];
    push_secret(&mut pairs, client.1);
    request_tokens(token_url, &pairs, clock).await
}

/// Ajoute `client_secret` au formulaire quand le client en a un
fn push_secret<'a>(pairs: &mut Vec<(&'a str, &'a str)>, secret: Option<&'a SecretValue>) {
    if let Some(secret) = secret {
        pairs.push(("client_secret", secret.expose()));
    }
}

/// Envoie la demande et convertit la réponse en jetons ou en erreur du contrat
async fn request_tokens(
    token_url: &str,
    pairs: &[(&str, &str)],
    clock: &dyn Clock,
) -> Result<TokenSet, MiraError> {
    let (status, body) = post_form(token_url, pairs).await?;
    if status == 200 {
        return parse_tokens(&body, clock);
    }
    Err(map_token_failure(status, &body))
}
