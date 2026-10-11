//! Décodage et vérification des échanges SASL PLAIN et XOAUTH2.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::credentials::Credentials;

/// Vérifie la réponse base64 ; renvoie l'utilisateur authentifié
pub(super) fn verify(mechanism: &str, answer: &str, credentials: &Credentials) -> Option<String> {
    let bytes = STANDARD.decode(answer.trim()).ok()?;
    match mechanism {
        "PLAIN" => verify_plain(&bytes, credentials),
        "XOAUTH2" => verify_xoauth2(&bytes, credentials),
        _ => None,
    }
}

/// PLAIN : `authzid NUL authcid NUL mot-de-passe`
fn verify_plain(bytes: &[u8], credentials: &Credentials) -> Option<String> {
    let mut parts = bytes.split(|byte| *byte == 0);
    let (authzid, authcid, password) = (parts.next()?, parts.next()?, parts.next()?);
    let chosen = if authcid.is_empty() { authzid } else { authcid };
    let user = String::from_utf8(chosen.to_vec()).ok()?;
    let password = String::from_utf8(password.to_vec()).ok()?;
    credentials.check_password(&user, &password).then_some(user)
}

/// XOAUTH2 : `user=<u> SOH auth=Bearer <jeton> SOH SOH`
fn verify_xoauth2(bytes: &[u8], credentials: &Credentials) -> Option<String> {
    let text = String::from_utf8(bytes.to_vec()).ok()?;
    let mut parts = text.split('\u{1}');
    let user = parts.next()?.strip_prefix("user=")?;
    let token = parts.next()?.strip_prefix("auth=Bearer ")?;
    credentials
        .check_access_token(user, token)
        .then(|| user.to_string())
}
