//! Identifiants simulés dont le rafraîchissement échoue par une coupure réseau, pas par un refus.

use std::sync::atomic::Ordering;

use mira_vocabulary::{ErrorCategory, ErrorCode, MiraError};

use super::credentials::FakeCredentials;

impl FakeCredentials {
    /// Identifiants périmés que le serveur SMTP refuse, dont le service de jetons coupe la connexion
    pub fn stale_login_with_cut_token_service() -> Self {
        let credentials = Self::valid();
        credentials.wrong_password.store(true, Ordering::SeqCst);
        let error = MiraError::new(
            ErrorCategory::Network,
            ErrorCode("network.connection_lost"),
            "échange interrompu avec le service d'autorisation",
        )
        .retryable(true);
        *credentials.refusal.lock().unwrap() = Some(error);
        credentials
    }
}
