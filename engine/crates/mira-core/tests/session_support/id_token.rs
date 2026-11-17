//! Jeton d'identité de test : trois segments, signature factice.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::json;

/// Réclamations d'un jeton d'identité de test
pub struct Claims<'a> {
    pub iss: &'a str,
    pub aud: &'a str,
    pub nonce: &'a str,
    pub exp: i64,
    pub iat: i64,
}

/// Construit le jeton `en-tête.charge.signature` ; l'adresse de courriel est fixe
pub fn make(claims: &Claims<'_>) -> String {
    let encode = |value: serde_json::Value| URL_SAFE_NO_PAD.encode(value.to_string());
    let header = encode(json!({"alg": "RS256", "typ": "JWT"}));
    let payload = encode(json!({
        "iss": claims.iss, "aud": claims.aud, "nonce": claims.nonce,
        "exp": claims.exp, "iat": claims.iat, "email": "abonne@example.test"
    }));
    format!("{header}.{payload}.SIGNATURE")
}
