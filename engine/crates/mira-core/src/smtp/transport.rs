//! Construction du transport `lettre` selon le chiffrement et le type d'identifiants.

use std::time::Duration;

use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use mira_vocabulary::error::codes::tls;
use mira_vocabulary::{ErrorCategory, MiraError, Security, ServerEndpoint};

use super::login::{SmtpCredential, SmtpLogin};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

/// Prépare le transport asynchrone vers le serveur, authentifié par `login`
pub(super) fn build_transport(
    endpoint: &ServerEndpoint,
    login: &SmtpLogin,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, MiraError> {
    let (secret, mechanisms) = match &login.credential {
        SmtpCredential::Password(value) => (value, vec![Mechanism::Plain, Mechanism::Login]),
        SmtpCredential::XOAuth2(token) => (token, vec![Mechanism::Xoauth2]),
    };
    let credentials = Credentials::new(login.user.clone(), secret.expose().to_string());
    let builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(endpoint.host.clone());
    Ok(builder
        .port(endpoint.port)
        .tls(tls_mode(endpoint)?)
        .credentials(credentials)
        .authentication(mechanisms)
        .timeout(Some(COMMAND_TIMEOUT))
        .build())
}

/// Traduit le chiffrement demandé en mode TLS de `lettre`
fn tls_mode(endpoint: &ServerEndpoint) -> Result<Tls, MiraError> {
    match endpoint.security {
        Security::Plaintext => Ok(Tls::None),
        Security::Tls => Ok(Tls::Wrapper(tls_parameters(endpoint)?)),
        Security::StartTls => Ok(Tls::Required(tls_parameters(endpoint)?)),
    }
}

/// Paramètres TLS vérifiant le certificat pour l'hôte du serveur
fn tls_parameters(endpoint: &ServerEndpoint) -> Result<TlsParameters, MiraError> {
    TlsParameters::new(endpoint.host.clone()).map_err(|_| {
        MiraError::new(
            ErrorCategory::Tls,
            tls::HANDSHAKE,
            "paramètres TLS SMTP invalides",
        )
    })
}
