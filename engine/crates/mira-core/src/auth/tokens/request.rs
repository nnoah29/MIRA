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
    let pairs = [
        ("grant_type", "authorization_code"),
        ("code", code.expose()),
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("code_verifier", verifier.expose()),
    ];
    request_tokens(token_url, &pairs, clock).await
}

/// Obtient un nouveau jeton d'accès à partir du jeton de rafraîchissement
pub async fn refresh_tokens(
    token_url: &str,
    client_id: &str,
    refresh_token: &SecretValue,
    clock: &dyn Clock,
) -> Result<TokenSet, MiraError> {
    let pairs = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token.expose()),
        ("client_id", client_id),
    ];
    request_tokens(token_url, &pairs, clock).await
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
