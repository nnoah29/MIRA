//! Mécanisme AUTH LOGIN : nom d'utilisateur puis mot de passe, chacun en base64.

use std::io;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use crate::smtp::session::Session;

/// Renvoie l'utilisateur authentifié après les deux défis Username et Password
pub(super) async fn authenticate(
    session: &mut Session,
    initial: Option<&str>,
) -> io::Result<Option<String>> {
    let user_b64 = super::obtain_response(session, initial, "VXNlcm5hbWU6").await?;
    let password_b64 = super::obtain_response(session, None, "UGFzc3dvcmQ6").await?;
    let (user, password) = (decode_text(&user_b64), decode_text(&password_b64));
    Ok(user.zip(password).and_then(|(user, password)| {
        session
            .credentials
            .check_password(&user, &password)
            .then_some(user)
    }))
}

/// Décode un texte UTF-8 en base64
fn decode_text(encoded: &str) -> Option<String> {
    String::from_utf8(STANDARD.decode(encoded).ok()?).ok()
}
