//! Politique de chiffrement : le clair n'est accepté que vers la boucle locale (AG-03).

use mira_vocabulary::{MiraError, Security, ServerEndpoint};

pub(crate) use crate::auth::loopback::is_loopback_host;
use crate::imap::errors::tls::plaintext_refused;

/// Refuse une connexion en clair vers un hôte qui n'est pas la boucle locale.
pub(crate) fn check_security(endpoint: &ServerEndpoint) -> Result<(), MiraError> {
    if endpoint.security == Security::Plaintext && !is_loopback_host(&endpoint.host) {
        return Err(plaintext_refused(&endpoint.host));
    }
    Ok(())
}
