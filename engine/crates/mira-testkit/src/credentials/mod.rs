//! Registre d'identifiants partagé par les faux serveurs (mots de passe et jetons d'accès).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

mod redacted_debug;

/// Registre partagé, clonable, des identifiants acceptés par les faux serveurs.
#[derive(Clone, Default)]
pub struct Credentials {
    table: Arc<Mutex<CredentialTable>>,
}

#[derive(Default)]
struct CredentialTable {
    passwords: HashMap<String, String>,
    access_tokens: HashMap<String, String>,
}

impl Credentials {
    /// Accède à la table en ignorant un éventuel empoisonnement du verrou
    fn table(&self) -> MutexGuard<'_, CredentialTable> {
        self.table
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Enregistre le mot de passe accepté pour un utilisateur
    pub fn add_password(&self, user: &str, password: &str) {
        self.table()
            .passwords
            .insert(user.to_string(), password.to_string());
    }

    /// Vérifie un couple utilisateur et mot de passe
    pub fn check_password(&self, user: &str, password: &str) -> bool {
        self.table()
            .passwords
            .get(user)
            .is_some_and(|known| known == password)
    }

    /// Enregistre un jeton d'accès valide pour un utilisateur
    pub fn add_access_token(&self, user: &str, token: &str) {
        self.table()
            .access_tokens
            .insert(token.to_string(), user.to_string());
    }

    /// Révoque un jeton d'accès (simule l'expiration ou la révocation)
    pub fn revoke_access_token(&self, token: &str) {
        self.table().access_tokens.remove(token);
    }

    /// Vérifie qu'un jeton d'accès est valide pour cet utilisateur
    pub fn check_access_token(&self, user: &str, token: &str) -> bool {
        self.table()
            .access_tokens
            .get(token)
            .is_some_and(|owner| owner == user)
    }
}
