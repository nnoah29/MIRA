//! Lecture des secrets d'un compte pour se connecter aux serveurs IMAP et SMTP.

use mira_vocabulary::error::codes::secrets;
use mira_vocabulary::{
    AccountId, AuthMethodKind, ErrorCategory, MiraError, ProfileId, SecretKind, SuggestedAction,
};

use super::credential_vault::CredentialVault;
use crate::ports::SecretValue;
use crate::smtp::{SmtpCredential, SmtpLogin};

/// Identifiant de connexion : utilisateur, secret (masqué) et méthode d'authentification.
#[derive(Debug, Clone)]
pub struct LoginSecret {
    pub user: String,
    pub secret: SecretValue,
    pub method: AuthMethodKind,
}

impl LoginSecret {
    /// Convertit en identifiants SMTP : XOAUTH2 pour OAuth2, mot de passe sinon
    pub fn into_smtp_login(self) -> SmtpLogin {
        let credential = match self.method {
            AuthMethodKind::OAuth2 => SmtpCredential::XOAuth2(self.secret),
            AuthMethodKind::AppPassword | AuthMethodKind::Password => {
                SmtpCredential::Password(self.secret)
            }
        };
        SmtpLogin {
            user: self.user,
            credential,
        }
    }
}

impl CredentialVault {
    /// Relit un secret du compte, ou rien s'il n'a jamais été rangé
    pub fn load_secret(
        &self,
        profile: &ProfileId,
        account: &AccountId,
        kind: SecretKind,
    ) -> Result<Option<SecretValue>, MiraError> {
        self.secrets.get(&Self::key(profile, account, kind))
    }

    /// Relit le secret de connexion : jeton d'accès pour OAuth2, mot de passe sinon
    pub fn load_login(
        &self,
        profile: &ProfileId,
        account: &AccountId,
        method: AuthMethodKind,
        user: &str,
    ) -> Result<LoginSecret, MiraError> {
        let kind = match method {
            AuthMethodKind::OAuth2 => SecretKind::AccessToken,
            AuthMethodKind::AppPassword | AuthMethodKind::Password => SecretKind::Password,
        };
        let secret = self.load_secret(profile, account, kind)?.ok_or_else(|| {
            MiraError::new(
                ErrorCategory::Secrets,
                secrets::SECRET_MISSING,
                "secret absent du trousseau",
            )
            .with_action(SuggestedAction::Reauthenticate)
        })?;
        Ok(LoginSecret {
            user: user.to_string(),
            secret,
            method,
        })
    }
}
