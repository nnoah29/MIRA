//! Coffre d'identifiants de test : trousseau en mémoire, base en mémoire, identifiants fixes.

use std::sync::Arc;

use mira_core::auth::{CredentialVault, TokenSet};
use mira_core::ports::{MemorySecretStore, SecretStore, SecretValue};
use mira_core::storage::Database;
use mira_vocabulary::{AccountId, ProfileId, SecretKind, SecretRef, Timestamp};

/// Coffre et ses deux dépendances, pour inspecter le trousseau et la base après coup.
pub struct VaultFixture {
    pub vault: CredentialVault,
    pub secrets: Arc<MemorySecretStore>,
    pub database: Arc<Database>,
}

impl VaultFixture {
    /// Expiration enregistrée en base pour le compte de test
    pub fn expiry(&self) -> Option<Timestamp> {
        let (_, account) = ids();
        self.database.read(|tx| tx.token_expiry(&account)).unwrap()
    }
}

/// Construit un coffre neuf au-dessus d'un trousseau et d'une base en mémoire
pub fn fixture() -> VaultFixture {
    let secrets = Arc::new(MemorySecretStore::new());
    let database = Arc::new(Database::open_in_memory().unwrap());
    let vault = CredentialVault::new(secrets.clone(), database.clone());
    VaultFixture {
        vault,
        secrets,
        database,
    }
}

/// Profil et compte utilisés par les tests du coffre
pub fn ids() -> (ProfileId, AccountId) {
    (ProfileId::from_raw("default"), AccountId::from_raw("acc-1"))
}

/// Jetons de test : jeton d'accès, jeton de rafraîchissement et expiration facultatifs
pub fn token_set(access: &str, refresh: Option<&str>, expires: Option<i64>) -> TokenSet {
    TokenSet {
        access_token: SecretValue::new(access),
        refresh_token: refresh.map(SecretValue::new),
        expires_at: expires.map(Timestamp::from_millis),
    }
}

/// Valeur d'un secret du compte de test dans le trousseau en mémoire
pub fn keyring_value(fixture: &VaultFixture, kind: SecretKind) -> Option<String> {
    let (profile, account) = ids();
    let key = SecretRef {
        profile,
        account,
        kind,
    };
    let value = fixture.secrets.get(&key).unwrap();
    value.map(|secret| secret.expose().to_string())
}
