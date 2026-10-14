//! Identifiants de test pour le faux serveur IMAP : mot de passe fixe, rafraîchissement programmable.

use std::sync::Mutex;

use mira_core::imap::{ImapCredential, ImapLogin};
use mira_core::ports::SecretValue;
use mira_core::sync::{BoxFuture, CredentialProvider};
use mira_vocabulary::{Account, MiraError};

/// Mot de passe courant et mot de passe que `refresh` met en place, s'il y en a un.
#[derive(Debug)]
pub struct TestCredentials {
    password: Mutex<String>,
    after_refresh: Option<String>,
}

impl TestCredentials {
    /// Identifiants valables pour `alice` sur le faux serveur
    pub fn valid() -> Self {
        Self {
            password: Mutex::new("secret".to_string()),
            after_refresh: None,
        }
    }

    /// Identifiants refusés par le serveur, sans rafraîchissement possible
    pub fn rejected() -> Self {
        Self {
            password: Mutex::new("wrong".to_string()),
            after_refresh: None,
        }
    }

    /// Identifiants refusés au départ, corrigés par le rafraîchissement
    pub fn repaired_by_refresh() -> Self {
        Self {
            password: Mutex::new("wrong".to_string()),
            after_refresh: Some("secret".to_string()),
        }
    }
}

impl CredentialProvider for TestCredentials {
    /// Présente `alice` avec le mot de passe courant
    fn imap_login<'a>(
        &'a self,
        _account: &'a Account,
    ) -> BoxFuture<'a, Result<ImapLogin, MiraError>> {
        let password = self.password.lock().unwrap().clone();
        Box::pin(async move {
            Ok(ImapLogin {
                user: "alice".to_string(),
                credential: ImapCredential::Password(SecretValue::new(&password)),
            })
        })
    }

    /// Met en place le mot de passe corrigé s'il y en a un
    fn refresh<'a>(&'a self, _account: &'a Account) -> BoxFuture<'a, Result<bool, MiraError>> {
        let renewed = self.after_refresh.clone();
        if let Some(good) = &renewed {
            *self.password.lock().unwrap() = good.clone();
        }
        Box::pin(async move { Ok(renewed.is_some()) })
    }
}
