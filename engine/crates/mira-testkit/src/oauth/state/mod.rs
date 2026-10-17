//! État du serveur d'autorisation : codes, jetons de rafraîchissement, jetons émis.

mod codes;
mod tokens;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub(crate) use codes::PendingCode;
pub(crate) use tokens::RefreshGrant;

/// Données mutables du serveur ; sans `Debug` car elles contiennent des jetons.
#[derive(Default)]
pub(crate) struct OAuthState {
    counter: u64,
    token_requests: usize,
    codes: HashMap<String, PendingCode>,
    refresh_tokens: HashMap<String, RefreshGrant>,
    issued_access: HashMap<String, Vec<String>>,
    required_client_secret: Option<String>,
}

/// Poignée clonable vers l'état partagé.
#[derive(Clone, Default)]
pub(crate) struct OAuthShared {
    inner: Arc<Mutex<OAuthState>>,
}

impl OAuthShared {
    /// Exécute une action sur l'état en ignorant un verrou empoisonné
    pub(crate) fn with<R>(&self, action: impl FnOnce(&mut OAuthState) -> R) -> R {
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        action(&mut guard)
    }
}

impl OAuthState {
    /// Produit un identifiant déterministe et unique grâce au compteur
    fn next_id(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("{prefix}-{:06}", self.counter)
    }

    /// Compte une requête reçue sur le point d'accès `/token`
    pub(crate) fn count_token_request(&mut self) {
        self.token_requests += 1;
    }

    /// Exige désormais ce `client_secret` sur le point d'accès `/token`
    pub(crate) fn require_client_secret(&mut self, secret: &str) {
        self.required_client_secret = Some(secret.to_string());
    }

    /// Indique si le secret présenté convient (toujours vrai si aucun n'est exigé)
    pub(crate) fn client_secret_accepted(&self, given: Option<&str>) -> bool {
        match &self.required_client_secret {
            Some(required) => given == Some(required.as_str()),
            None => true,
        }
    }

    /// Nombre de requêtes `/token` reçues jusqu'ici
    pub(crate) fn token_requests(&self) -> usize {
        self.token_requests
    }
}
