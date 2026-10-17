//! Mécanisme AUTH XOAUTH2 : `user=<u> SOH auth=Bearer <jeton> SOH SOH` en base64.

use std::io;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use crate::smtp::session::Session;

/// Renvoie l'utilisateur authentifié si le jeton d'accès est enregistré
pub(super) async fn authenticate(
    session: &mut Session,
    initial: Option<&str>,
) -> io::Result<Option<String>> {
    let response = super::obtain_response(session, initial, "").await?;
    let accepted = decode(&response)
        .filter(|(user, token)| session.credentials.check_access_token(user, token));
    Ok(accepted.map(|(user, _)| user))
}

/// Décode la réponse XOAUTH2 en couple utilisateur et jeton
fn decode(response: &str) -> Option<(String, String)> {
    let text = String::from_utf8(STANDARD.decode(response).ok()?).ok()?;
    let mut fields = text.split('\u{1}');
    let user = fields.next()?.strip_prefix("user=")?;
    let token = fields.next()?.strip_prefix("auth=Bearer ")?;
    Some((user.to_string(), token.to_string()))
}
