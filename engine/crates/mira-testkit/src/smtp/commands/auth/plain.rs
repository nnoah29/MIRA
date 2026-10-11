//! Mécanisme AUTH PLAIN (RFC 4616) : `authzid NUL authcid NUL mot-de-passe` en base64.

use std::io;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use crate::smtp::session::Session;

/// Renvoie l'utilisateur authentifié si le mot de passe est connu
pub(super) async fn authenticate(
    session: &mut Session,
    initial: Option<&str>,
) -> io::Result<Option<String>> {
    let response = super::obtain_response(session, initial, "").await?;
    let accepted = decode(&response)
        .filter(|(user, password)| session.credentials.check_password(user, password));
    Ok(accepted.map(|(user, _)| user))
}

/// Décode la réponse PLAIN en couple utilisateur et mot de passe
fn decode(response: &str) -> Option<(String, String)> {
    let bytes = STANDARD.decode(response).ok()?;
    let mut parts = bytes.split(|byte| *byte == 0);
    parts.next()?;
    let user = String::from_utf8(parts.next()?.to_vec()).ok()?;
    let password = String::from_utf8(parts.next()?.to_vec()).ok()?;
    parts.next().is_none().then_some((user, password))
}
