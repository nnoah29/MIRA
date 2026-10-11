//! Négociation TLS avec rustls et les racines de confiance de `webpki-roots`.

use std::sync::Arc;

use mira_vocabulary::error::codes::tls;
use mira_vocabulary::{ErrorCategory, MiraError};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsConnector;

use crate::imap::errors::tls::from_handshake;
use crate::imap::protocol::wire::BoxedTransport;

/// Erreur de configuration TLS interne, sans détail sensible.
fn config_error() -> MiraError {
    MiraError::new(
        ErrorCategory::Tls,
        tls::HANDSHAKE,
        "configuration TLS impossible",
    )
}

/// Construit la configuration cliente : racines publiques, versions sûres par défaut.
fn client_config() -> Result<ClientConfig, MiraError> {
    let roots = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| config_error())?;
    Ok(builder.with_root_certificates(roots).with_no_client_auth())
}

/// Enveloppe un transport dans une session TLS vérifiant le nom du serveur.
pub(crate) async fn wrap_tls<T>(stream: T, host: &str) -> Result<BoxedTransport, MiraError>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let name = ServerName::try_from(host.to_string()).map_err(|_| config_error())?;
    let connector = TlsConnector::from(Arc::new(client_config()?));
    match connector.connect(name, stream).await {
        Ok(secured) => Ok(Box::new(secured)),
        Err(error) => Err(from_handshake(&error)),
    }
}
