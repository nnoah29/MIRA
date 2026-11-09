//! Échange du code d'une connexion par navigateur contre une clé d'API (JSON, sans identifiant de client).

use mira_vocabulary::MiraError;
use serde_json::json;

use super::failure::map_token_failure;
use super::http::post_json;
use super::response::parse_key;
use crate::auth::flow::AuthorizationCode;
use crate::ports::SecretValue;

/// Présente le code et le vérificateur PKCE (S256) au point d'échange et rend la clé reçue
pub async fn exchange_key_code(
    exchange_url: &str,
    code: &AuthorizationCode,
    verifier: &SecretValue,
) -> Result<SecretValue, MiraError> {
    let body = json!({
        "code": code.expose(),
        "code_verifier": verifier.expose(),
        "code_challenge_method": "S256",
    });
    let (status, answer) = post_json(exchange_url, &body).await?;
    if (200..300).contains(&status) {
        return parse_key(&answer);
    }
    Err(map_token_failure(status, &answer))
}
