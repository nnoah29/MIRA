//! Secret d'application d'un flux : présenté avec l'identifiant lors de l'échange du code.

use super::pending::PendingFlow;
use crate::ports::SecretValue;

impl PendingFlow {
    /// Fixe le secret d'application à présenter à l'échange du code (absent pour un client public)
    pub fn set_client_secret(&mut self, secret: Option<SecretValue>) {
        self.client_secret = secret;
    }
}
