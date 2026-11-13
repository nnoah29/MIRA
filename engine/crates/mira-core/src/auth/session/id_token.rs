//! Contrôle d'un jeton d'identité reçu du point de jeton : émetteur, public, nonce et dates.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{ErrorCategory, MiraError};
use serde_json::Value;

const CLOCK_SKEW: i64 = 300;

/// Vérifie `iss`, `aud`, `nonce`, `exp` et `iat` ; rend l'adresse de courriel si le jeton en porte une
pub(super) fn validate_id_token(
    token: &str,
    issuer: &str,
    audience: &str,
    nonce: &str,
    now: i64,
) -> Result<Option<String>, MiraError> {
    let claims = payload(token).ok_or_else(rejected)?;
    let text = |name: &str| claims.get(name).and_then(Value::as_str);
    let number = |name: &str| claims.get(name).and_then(Value::as_i64);
    let fresh = number("exp").is_some_and(|exp| exp > now - CLOCK_SKEW)
        && number("iat").is_some_and(|iat| iat <= now + CLOCK_SKEW);
    let valid = text("iss") == Some(issuer)
        && audience_matches(claims.get("aud"), audience)
        && text("nonce") == Some(nonce)
        && fresh;
    if !valid {
        return Err(rejected());
    }
    Ok(text("email").map(str::to_string))
}

/// Contenu du deuxième segment du jeton, décodé ; la signature n'est pas contrôlée (canal TLS direct)
fn payload(token: &str) -> Option<Value> {
    let segment = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(segment).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Dit si `aud` (texte ou liste) désigne le client attendu
fn audience_matches(aud: Option<&Value>, expected: &str) -> bool {
    match aud {
        Some(Value::String(one)) => one == expected,
        Some(Value::Array(list)) => list.iter().any(|item| item.as_str() == Some(expected)),
        _ => false,
    }
}

/// Erreur d'un jeton d'identité inattendu, sans en reprendre le contenu
fn rejected() -> MiraError {
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::REJECTED,
        "jeton d'identité inattendu",
    )
}
