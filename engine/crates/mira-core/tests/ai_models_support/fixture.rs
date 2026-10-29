//! Moteur ouvert avec un fournisseur de test, faux serveur d'IA et accès direct à la base du profil.

use std::sync::Arc;

use mira_core::ai::AiProviderRegistry;
use mira_core::api::Engine;
use mira_core::ports::{MemorySecretStore, SecretStore, SecretValue};
use mira_core::profile::ProfilePaths;
use mira_core::storage::Database;
use mira_testkit::ai::FakeAiServer;
use mira_vocabulary::{
    AiConnection, AiConnectionId, AiProviderId, AiSecretRef, CONTRACT_VERSION, ProfileId,
};
use tempfile::TempDir;

use super::TestProvider;
use crate::support_api::config_with_secrets;

/// Moteur de test, sa base relue par une seconde ouverture, son trousseau et le faux serveur d'IA.
pub struct ModelsFixture {
    pub home: TempDir,
    pub engine: Engine,
    pub db: Database,
    pub secrets: Arc<MemorySecretStore>,
    pub server: FakeAiServer,
}

impl ModelsFixture {
    /// Ouvre un moteur dont le registre d'IA contient ce fournisseur, dont l'adresse est celle du faux serveur
    pub async fn start(mut provider: TestProvider) -> Self {
        let server = FakeAiServer::start().await.unwrap();
        provider.base_url = server.base_url();
        let home = tempfile::tempdir().unwrap();
        let secrets = Arc::new(MemorySecretStore::new());
        let mut config = config_with_secrets(home.path(), Arc::clone(&secrets));
        let mut registry = AiProviderRegistry::new();
        registry.register(Arc::new(provider));
        config.ai_registry = registry;
        let paths = ProfilePaths::resolve(&config.profile, &config.environment).unwrap();
        let engine = Engine::open(config, CONTRACT_VERSION).unwrap();
        let db = Database::open(&paths.database).unwrap();
        Self {
            home,
            engine,
            db,
            secrets,
            server,
        }
    }

    /// Range une clé d'API pour cette connexion dans le trousseau
    pub fn store_key(&self, connection: &str, key: &str) {
        let location = AiSecretRef {
            profile: ProfileId::from_raw("default"),
            connection: AiConnectionId::from_raw(connection),
        };
        self.secrets.put(&location, &SecretValue::new(key)).unwrap();
    }

    /// Insère en base une connexion de ce fournisseur pointant sur `base_url` et la relit
    pub fn add_connection(&self, provider: &str, base_url: Option<String>) -> AiConnection {
        let connection = AiConnection {
            id: AiConnectionId::from_raw("conn-1"),
            provider: AiProviderId(provider.to_string()),
            label: "Essai".to_string(),
            base_url,
            default_model: Some("model-a".to_string()),
            is_default: false,
            has_key: false,
            consent_at: None,
            models: None,
        };
        self.db
            .write(|tx| tx.insert_ai_connection(&connection, 1))
            .unwrap();
        connection
    }

    /// Relit la connexion telle qu'elle est en base
    pub fn stored(&self, connection: &AiConnection) -> AiConnection {
        self.db
            .read(|tx| tx.require_ai_connection(&connection.id))
            .unwrap()
    }
}
