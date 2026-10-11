//! Codes d'autorisation à usage unique liés à un défi PKCE et à un utilisateur.

use super::OAuthState;

/// Code émis en attente d'échange contre des jetons.
pub(crate) struct PendingCode {
    pub(crate) user: String,
    pub(crate) client_id: String,
    pub(crate) redirect_uri: String,
    pub(crate) challenge: String,
}

impl OAuthState {
    /// Mémorise un code d'autorisation et renvoie sa valeur
    pub(crate) fn store_code(&mut self, pending: PendingCode) -> String {
        let code = self.next_id("code");
        self.codes.insert(code.clone(), pending);
        code
    }

    /// Retire le code : il ne peut servir qu'une fois, même si l'échange échoue ensuite
    pub(crate) fn take_code(&mut self, code: &str) -> Option<PendingCode> {
        self.codes.remove(code)
    }
}
