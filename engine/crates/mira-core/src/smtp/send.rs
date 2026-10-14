//! Envoi d'un message : contrôle AG-03, enveloppe, transport, conversion des erreurs.

use std::str::FromStr;
use std::time::Duration;

use lettre::AsyncTransport;
use lettre::address::{Address, Envelope};
use mira_vocabulary::error::codes::{invalid_input, network, tls};
use mira_vocabulary::{ErrorCategory, MiraError, Security, ServerEndpoint};

use super::errors::map_error;
use super::login::{SmtpCredential, SmtpLogin};
use super::outgoing::OutgoingMessage;
use super::reply_codes::fail;
use super::transport::build_transport;
use crate::auth::loopback::is_loopback_host;

const SEND_TIMEOUT: Duration = Duration::from_secs(120);

/// Remet le message au serveur SMTP ; `network.connection_lost` signale une issue incertaine
pub async fn send_message(
    endpoint: &ServerEndpoint,
    login: &SmtpLogin,
    message: &OutgoingMessage,
) -> Result<(), MiraError> {
    send_within(endpoint, login, message, SEND_TIMEOUT).await
}

/// Comme `send_message` avec un délai global ; un dépassement est une issue incertaine, jamais un délai passager
pub async fn send_within(
    endpoint: &ServerEndpoint,
    login: &SmtpLogin,
    message: &OutgoingMessage,
    limit: Duration,
) -> Result<(), MiraError> {
    refuse_remote_plaintext(endpoint)?;
    let envelope = build_envelope(message)?;
    let transport = build_transport(endpoint, login)?;
    let xoauth = matches!(login.credential, SmtpCredential::XOAuth2(_));
    let delivery = transport.send_raw(&envelope, &message.raw);
    match tokio::time::timeout(limit, delivery).await {
        Ok(outcome) => outcome
            .map(|_| ())
            .map_err(|error| map_error(&error, xoauth)),
        Err(_) => Err(connection_lost()),
    }
}

/// Erreur d'une connexion devenue muette : l'issue de l'envoi est incertaine
fn connection_lost() -> MiraError {
    let message = "serveur SMTP muet, l'issue de l'envoi est incertaine";
    fail(
        ErrorCategory::Network,
        network::CONNECTION_LOST,
        message,
        false,
    )
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
