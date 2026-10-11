//! Envoi d'un message : contrôle AG-03, enveloppe, transport, conversion des erreurs.

use std::str::FromStr;

use lettre::AsyncTransport;
use lettre::address::{Address, Envelope};
use mira_vocabulary::error::codes::{invalid_input, tls};
use mira_vocabulary::{ErrorCategory, MiraError, Security, ServerEndpoint};

use super::errors::map_error;
use super::login::{SmtpCredential, SmtpLogin};
use super::outgoing::OutgoingMessage;
use super::transport::build_transport;
use crate::auth::loopback::is_loopback_host;

/// Remet le message au serveur SMTP ; `network.connection_lost` signale une issue incertaine
pub async fn send_message(
    endpoint: &ServerEndpoint,
    login: &SmtpLogin,
    message: &OutgoingMessage,
) -> Result<(), MiraError> {
    refuse_remote_plaintext(endpoint)?;
    let envelope = build_envelope(message)?;
    let transport = build_transport(endpoint, login)?;
    let xoauth = matches!(login.credential, SmtpCredential::XOAuth2(_));
    let outcome = transport.send_raw(&envelope, &message.raw).await;
    outcome
        .map(|_| ())
        .map_err(|error| map_error(&error, xoauth))
}

/// Refuse le clair vers un hôte qui n'est pas la boucle locale (AG-03)
fn refuse_remote_plaintext(endpoint: &ServerEndpoint) -> Result<(), MiraError> {
    if endpoint.security == Security::Plaintext && !is_loopback_host(&endpoint.host) {
        return Err(MiraError::new(
            ErrorCategory::Tls,
            tls::PLAINTEXT_REFUSED,
            "SMTP sans TLS refusé hors boucle locale",
        ));
    }
    Ok(())
}

/// Construit l'enveloppe SMTP à partir de l'expéditeur et des destinataires
fn build_envelope(message: &OutgoingMessage) -> Result<Envelope, MiraError> {
    let invalid = || {
        MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::ADDRESS,
            "adresse d'enveloppe invalide",
        )
    };
    let from = Address::from_str(&message.from).map_err(|_| invalid())?;
    let recipients = message
        .recipients
        .iter()
        .map(|address| Address::from_str(address).map_err(|_| invalid()))
        .collect::<Result<Vec<_>, _>>()?;
    Envelope::new(Some(from), recipients).map_err(|_| invalid())
}
