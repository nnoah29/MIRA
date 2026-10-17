//! Identifiants simulés du rejeu : jeton expiré ou mot de passe refusé, rafraîchissement accepté ou refusé.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use mira_vocabulary::{ErrorCategory, ErrorCode, MiraError};

/// Identifiants de test : le jeton est « expiré » à la demande, son rafraîchissement peut être refusé.
#[derive(Debug)]
pub struct FakeCredentials {
    pub(super) expired: AtomicBool,
    pub(super) wrong_password: AtomicBool,
    pub(super) renewals: AtomicU32,
    pub(super) refusal: Mutex<Option<MiraError>>,
}

impl FakeCredentials {
    /// Identifiants valables, rien à rafraîchir
    pub fn valid() -> Self {
        Self {
            expired: AtomicBool::new(false),
            wrong_password: AtomicBool::new(false),
            renewals: AtomicU32::new(0),
            refusal: Mutex::new(None),
        }
    }

    /// Jeton expiré dont le rafraîchissement échoue : le service l'a révoqué
    pub fn expired_and_revoked() -> Self {
        let credentials = Self::valid();
        credentials.expired.store(true, Ordering::SeqCst);
        credentials.refuse_renewal("authentication.token_revoked");
        credentials
    }

    /// Mot de passe refusé par le serveur SMTP, rafraîchissement impossible
    pub fn rejected_password() -> Self {
        let credentials = Self::valid();
        credentials.wrong_password.store(true, Ordering::SeqCst);
        credentials.refuse_renewal("authentication.rejected");
        credentials
    }

    /// Identifiants périmés que le serveur SMTP refuse mais qu'un rafraîchissement répare
    pub fn stale_login() -> Self {
        let credentials = Self::valid();
        credentials.wrong_password.store(true, Ordering::SeqCst);
        credentials
    }

    /// Fait échouer tout rafraîchissement avec une erreur d'authentification de ce code
    pub fn refuse_renewal(&self, code: &'static str) {
        let error = MiraError::new(
            ErrorCategory::Authentication,
            ErrorCode(code),
            "rafraîchissement refusé",
        );
        *self.refusal.lock().unwrap() = Some(error);
    }

    /// Nombre de rafraîchissements demandés jusqu'ici
    pub fn renewals(&self) -> u32 {
        self.renewals.load(Ordering::SeqCst)
    }
}
