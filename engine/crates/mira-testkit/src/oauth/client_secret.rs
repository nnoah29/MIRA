//! Pilotage du faux serveur : exiger un `client_secret` (clients « bureau » de certains services).

use super::server::FakeOAuthServer;

impl FakeOAuthServer {
    /// Refuse (`invalid_client`, 401) toute requête `/token` qui ne présente pas ce secret
    pub fn require_client_secret(&self, secret: &str) {
        self.context
            .shared
            .with(|state| state.require_client_secret(secret));
    }
}
