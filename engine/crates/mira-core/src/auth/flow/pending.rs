//! Flux d'autorisation en cours : écouteur local, vérificateur PKCE et état anti-CSRF.

use tokio::net::TcpListener;

use super::phase::FlowPhase;
use crate::ports::SecretValue;

/// Flux démarré ; le vérificateur et l'état ne sont jamais affichés par `Debug`.
pub struct PendingFlow {
    pub(super) authorize_url: String,
    pub(super) redirect_uri: String,
    pub(super) state: String,
    pub(super) verifier: SecretValue,
    pub(super) client_id: String,
    pub(super) token_url: String,
    pub(super) listener: Option<TcpListener>,
    pub(super) phase: FlowPhase,
}

impl PendingFlow {
    /// URL à ouvrir dans le navigateur de l'utilisateur
    pub fn authorize_url(&self) -> &str {
        &self.authorize_url
    }

    /// Adresse de redirection en boucle locale, port éphémère compris
    pub fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }

    /// Étape courante du flux
    pub fn phase(&self) -> FlowPhase {
        self.phase
    }

    /// Identifiant public du client OAuth utilisé pour ce flux
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Adresse du point d'accès de jetons du service
    pub fn token_url(&self) -> &str {
        &self.token_url
    }

    /// Vérificateur PKCE, à présenter lors de l'échange du code
    pub fn verifier(&self) -> &SecretValue {
        &self.verifier
    }
}
