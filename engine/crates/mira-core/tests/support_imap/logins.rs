//! Identifiants et connexions préparées pour les tests IMAP.

use mira_core::imap::{ImapClient, ImapCredential, ImapLogin};
use mira_core::ports::SecretValue;

use super::env::{Env, endpoint};

/// Connexion de `alice` avec son mot de passe
pub fn password_login(password: &str) -> ImapLogin {
    ImapLogin {
        user: "alice".to_string(),
        credential: ImapCredential::Password(SecretValue::new(password)),
    }
}

/// Connexion de `alice` avec un jeton d'accès
pub fn token_login(token: &str) -> ImapLogin {
    ImapLogin {
        user: "alice".to_string(),
        credential: ImapCredential::XOAuth2(SecretValue::new(token)),
    }
}

/// Client authentifié en tant qu'`alice`
pub async fn connected(env: &Env) -> ImapClient {
    ImapClient::connect(&endpoint(env), &password_login("secret"))
        .await
        .unwrap()
}
