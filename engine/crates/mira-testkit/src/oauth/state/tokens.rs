//! Jetons d'accès et de rafraîchissement émis, avec leur révocation.

use super::OAuthState;
use crate::credentials::Credentials;

/// Jeton de rafraîchissement : utilisateur propriétaire et client concerné.
#[derive(Clone)]
pub(crate) struct RefreshGrant {
    pub(crate) user: String,
    pub(crate) client_id: String,
}

impl OAuthState {
    /// Émet un jeton d'accès et l'enregistre dans les identifiants partagés
    pub(crate) fn issue_access(&mut self, user: &str, credentials: &Credentials) -> String {
        let token = self.next_id("access");
        credentials.add_access_token(user, &token);
        let issued = self.issued_access.entry(user.to_string()).or_default();
        issued.push(token.clone());
        token
    }

    /// Émet un jeton de rafraîchissement pour cet utilisateur et ce client
    pub(crate) fn issue_refresh(&mut self, user: &str, client_id: &str) -> String {
        let token = self.next_id("refresh");
        let grant = RefreshGrant {
            user: user.to_string(),
            client_id: client_id.to_string(),
        };
        self.refresh_tokens.insert(token.clone(), grant);
        token
    }

    /// Retrouve l'autorisation liée à un jeton de rafraîchissement
    pub(crate) fn refresh_grant(&self, token: &str) -> Option<RefreshGrant> {
        self.refresh_tokens.get(token).cloned()
    }

    /// Rend le rafraîchissement impossible pour cet utilisateur
    pub(crate) fn revoke_refresh(&mut self, user: &str) {
        self.refresh_tokens.retain(|_, grant| grant.user != user);
    }

    /// Révoque les jetons d'accès émis pour cet utilisateur
    pub(crate) fn expire_access(&mut self, user: &str, credentials: &Credentials) {
        for token in self.issued_access.remove(user).unwrap_or_default() {
            credentials.revoke_access_token(&token);
        }
    }
}
