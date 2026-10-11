//! Adresses du serveur et pilotage par les tests : expiration et révocation.

use super::server::FakeOAuthServer;

impl FakeOAuthServer {
    /// Port d'écoute éphémère
    pub fn port(&self) -> u16 {
        self.port
    }

    /// URL du point d'accès d'autorisation
    pub fn authorize_url(&self) -> String {
        format!("http://127.0.0.1:{}/authorize", self.port)
    }

    /// URL du point d'accès de jetons
    pub fn token_url(&self) -> String {
        format!("http://127.0.0.1:{}/token", self.port)
    }

    /// Rend le rafraîchissement impossible pour cet utilisateur
    pub fn revoke_refresh_tokens(&self, user: &str) {
        self.context.shared.with(|state| state.revoke_refresh(user));
    }

    /// Révoque les jetons d'accès émis pour cet utilisateur ; le rafraîchissement reste possible
    pub fn expire_access_tokens(&self, user: &str) {
        let credentials = &self.context.credentials;
        self.context
            .shared
            .with(|state| state.expire_access(user, credentials));
    }

    /// Nombre de requêtes reçues sur `/token`
    pub fn token_requests(&self) -> usize {
        self.context.shared.with(|state| state.token_requests())
    }
}
