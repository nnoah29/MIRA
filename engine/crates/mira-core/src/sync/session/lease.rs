//! Client IMAP prêté par le budget : le permis est rendu quand le client est détruit.

use std::ops::{Deref, DerefMut};

use mira_vocabulary::MiraError;
use tokio::sync::OwnedSemaphorePermit;

use crate::imap::ImapClient;

/// Connexion IMAP qui garde son permis du budget du compte tant qu'elle vit.
#[derive(Debug)]
pub struct LeasedClient {
    client: ImapClient,
    _permit: OwnedSemaphorePermit,
}

impl LeasedClient {
    /// Lie un client à son permis.
    pub fn new(client: ImapClient, permit: OwnedSemaphorePermit) -> Self {
        Self {
            client,
            _permit: permit,
        }
    }

    /// Ferme la session puis rend le permis.
    pub async fn logout(self) -> Result<(), MiraError> {
        self.client.logout().await
    }
}

impl Deref for LeasedClient {
    type Target = ImapClient;

    /// Accès en lecture au client prêté
    fn deref(&self) -> &ImapClient {
        &self.client
    }
}

impl DerefMut for LeasedClient {
    /// Accès en écriture au client prêté
    fn deref_mut(&mut self) -> &mut ImapClient {
        &mut self.client
    }
}
