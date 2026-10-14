//! Port d'identifiants du rejeu pour les identifiants simulés : connexion SMTP et rafraîchissement.

use std::sync::atomic::Ordering;

use mira_core::actions::{BoxFuture, ReplayCredentials};
use mira_core::ports::SecretValue;
use mira_core::smtp::{SmtpCredential, SmtpLogin};
use mira_vocabulary::{AccountId, MiraError};

use super::credentials::FakeCredentials;
use crate::support_smtp::server::{PASSWORD, USER};

impl FakeCredentials {
    /// Tente un rafraîchissement : refusé si une erreur est programmée
    fn renew(&self) -> Result<(), MiraError> {
        self.renewals.fetch_add(1, Ordering::SeqCst);
        match self.refusal.lock().unwrap().clone() {
            Some(error) => Err(error),
            None => {
                self.expired.store(false, Ordering::SeqCst);
                self.wrong_password.store(false, Ordering::SeqCst);
                Ok(())
            }
        }
    }
}

impl ReplayCredentials for FakeCredentials {
    /// Connexion par mot de passe au faux serveur SMTP
    fn smtp_login(&self, _account: &AccountId) -> Result<SmtpLogin, MiraError> {
        let wrong = self.wrong_password.load(Ordering::SeqCst);
        let password = if wrong { "mauvais" } else { PASSWORD };
        Ok(SmtpLogin {
            user: USER.to_string(),
            credential: SmtpCredential::Password(SecretValue::new(password)),
        })
    }

    /// Rafraîchit seulement si le jeton est marqué expiré
    fn renew_if_expired<'a>(
        &'a self,
        _account: &'a AccountId,
    ) -> BoxFuture<'a, Result<(), MiraError>> {
        Box::pin(async move {
            if self.expired.load(Ordering::SeqCst) {
                return self.renew();
            }
            Ok(())
        })
    }

    /// Rafraîchit sans condition
    fn renew_now<'a>(&'a self, _account: &'a AccountId) -> BoxFuture<'a, Result<(), MiraError>> {
        Box::pin(async move { self.renew() })
    }
}
