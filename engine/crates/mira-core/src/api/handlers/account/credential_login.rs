//! Identifiants IMAP construits à partir du secret saisi.

use mira_vocabulary::EmailAddress;

use crate::api::contract::CredentialInput;
use crate::imap::{ImapCredential, ImapLogin};
use crate::ports::SecretValue;

/// Valeur secrète contenue dans le secret saisi
pub(super) fn secret_value_of(credential: CredentialInput) -> SecretValue {
    match credential {
        CredentialInput::Password(input) | CredentialInput::AppPassword(input) => {
            input.into_secret_value()
        }
    }
}

/// Identifiants IMAP d'un mot de passe : l'utilisateur est l'adresse complète
pub(super) fn password_login(address: &EmailAddress, secret: SecretValue) -> ImapLogin {
    ImapLogin {
        user: address.as_str().to_string(),
        credential: ImapCredential::Password(secret),
    }
}
