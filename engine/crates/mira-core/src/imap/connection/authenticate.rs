//! Authentification : SASL PLAIN ou LOGIN pour un mot de passe, XOAUTH2 pour un jeton.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use mira_vocabulary::MiraError;

use super::capabilities::fetch_capabilities;
use crate::imap::errors::status::Operation;
use crate::imap::parse::capability::capabilities_of;
use crate::imap::protocol::quoting::quote;
use crate::imap::protocol::response::Response;
use crate::imap::protocol::wire::Wire;
use crate::imap::types::{ImapCredential, ImapLogin};

/// Réponse initiale SASL PLAIN encodée en base64.
fn plain_answer(user: &str, password: &str) -> String {
    STANDARD.encode(format!("\0{user}\0{password}"))
}

/// Réponse initiale SASL XOAUTH2 encodée en base64.
fn xoauth2_answer(user: &str, token: &str) -> String {
    STANDARD.encode(format!("user={user}\u{1}auth=Bearer {token}\u{1}\u{1}"))
}

/// Envoie la commande d'authentification adaptée au secret et aux capacités.
async fn send_credentials(
    wire: &mut Wire,
    login: &ImapLogin,
    capabilities: &[String],
) -> Result<Response, MiraError> {
    let user = login.user.as_str();
    match &login.credential {
        ImapCredential::XOAuth2(token) => {
            let answers = [xoauth2_answer(user, token.expose())];
            wire.command_with_answers("AUTHENTICATE XOAUTH2", &answers)
                .await
        }
        ImapCredential::Password(secret) if capabilities.iter().any(|c| c == "AUTH=PLAIN") => {
            let answers = [plain_answer(user, secret.expose())];
            wire.command_with_answers("AUTHENTICATE PLAIN", &answers)
                .await
        }
        ImapCredential::Password(secret) => {
            let line = format!("LOGIN {} {}", quote(user)?, quote(secret.expose())?);
            wire.command(&line).await
        }
    }
}

/// Authentifie la connexion et rend les capacités annoncées après authentification.
pub(crate) async fn authenticate(
    wire: &mut Wire,
    login: &ImapLogin,
    capabilities: &[String],
) -> Result<Vec<String>, MiraError> {
    let operation = match login.credential {
        ImapCredential::XOAuth2(_) => Operation::TokenLogin,
        ImapCredential::Password(_) => Operation::PasswordLogin,
    };
    let response = send_credentials(wire, login, capabilities)
        .await?
        .require_ok(operation)?;
    let announced = response
        .status
        .code
        .as_ref()
        .filter(|code| code.name == "CAPABILITY");
    match announced {
        Some(code) => Ok(code
            .args
            .split_whitespace()
            .map(str::to_ascii_uppercase)
            .collect()),
        None => match capabilities_of(&response.untagged) {
            Some(list) => Ok(list),
            None => fetch_capabilities(wire).await,
        },
    }
}
