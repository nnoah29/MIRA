//! Connexion complète d'un client : ouverture, authentification, capacités.

use mira_vocabulary::{MiraError, ServerEndpoint};

use super::authenticate::authenticate;
use super::open::open_wire;
use super::policy::check_security;
use crate::imap::client::ImapClient;
use crate::imap::types::{ImapLogin, ImapOptions};

impl ImapClient {
    /// Se connecte et s'authentifie avec les réglages par défaut (délai de 30 secondes).
    pub async fn connect(
        endpoint: &ServerEndpoint,
        login: &ImapLogin,
    ) -> Result<ImapClient, MiraError> {
        Self::connect_with(endpoint, login, ImapOptions::default()).await
    }

    /// Se connecte et s'authentifie avec des réglages explicites ; le clair est refusé hors boucle locale.
    pub async fn connect_with(
        endpoint: &ServerEndpoint,
        login: &ImapLogin,
        options: ImapOptions,
    ) -> Result<ImapClient, MiraError> {
        check_security(endpoint)?;
        let mut opened = open_wire(endpoint, options.timeout).await?;
        let capabilities = if opened.preauthenticated {
            opened.capabilities
        } else {
            authenticate(&mut opened.wire, login, &opened.capabilities).await?
        };
        Ok(ImapClient::from_parts(opened.wire, capabilities))
    }
}
