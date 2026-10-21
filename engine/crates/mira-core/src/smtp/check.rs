//! Test de connexion SMTP : connexion, chiffrement et authentification, sans envoyer de message.

use std::time::Duration;

use mira_vocabulary::error::codes::network;
use mira_vocabulary::{ErrorCategory, MiraError, ServerEndpoint};

use super::errors::map_error;
use super::login::{SmtpCredential, SmtpLogin};
use super::reply_codes::fail;
use super::send::refuse_remote_plaintext;
use super::transport::build_transport;

const CHECK_TIMEOUT: Duration = Duration::from_secs(30);

/// Se connecte au serveur SMTP et s'y authentifie, puis ferme la session sans rien envoyer
pub async fn check_smtp_login(
    endpoint: &ServerEndpoint,
    login: &SmtpLogin,
) -> Result<(), MiraError> {
    refuse_remote_plaintext(endpoint)?;
    let transport = build_transport(endpoint, login)?;
    let xoauth = matches!(login.credential, SmtpCredential::XOAuth2(_));
    match tokio::time::timeout(CHECK_TIMEOUT, transport.test_connection()).await {
        Ok(Ok(true)) => Ok(()),
        Ok(Ok(false)) | Err(_) => Err(unreachable_smtp()),
        Ok(Err(error)) => Err(map_error(&error, xoauth)),
    }
}

/// Erreur d'un serveur SMTP qui ne répond pas : rien n'est parti, l'essai peut être refait
fn unreachable_smtp() -> MiraError {
    let message = "serveur SMTP injoignable";
    fail(ErrorCategory::Network, network::UNREACHABLE, message, true)
}
