//! Moteur ouvert avec la configuration de production, le faux serveur d'IA et un trousseau en mémoire.

use std::sync::Arc;

use mira_assembly::api::Engine;
use mira_assembly::auth::OAuthClientConfig;
use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ProfileId};
use mira_assembly::ports::MemorySecretStore;
use mira_assembly::production_config_with_oauth;
use mira_assembly::profile::ProfileEnvironment;
use mira_assembly::sync::BackgroundSync;
use mira_testkit::ai::FakeAiServer;
use tempfile::TempDir;

pub const KEY: &str = "sk-E2E-SECRET-KEY-42";

/// Moteur assemblé comme en production, sauf trousseau, dossier de données et synchronisation de fond.
pub struct Fixture {
    pub home: TempDir,
    pub engine: Engine,
    pub secrets: Arc<MemorySecretStore>,
    pub server: FakeAiServer,
}

/// Ouvre le moteur de production sur un profil jetable, avec le faux serveur d'IA sur la boucle locale
pub async fn start() -> Fixture {
    let server = FakeAiServer::start().await.unwrap();
    let home = tempfile::tempdir().unwrap();
    let secrets = Arc::new(MemorySecretStore::new());
    let oauth = OAuthClientConfig::default();
    let mut config = production_config_with_oauth(ProfileId::from_raw("default"), oauth);
    config.secrets = secrets.clone();
    config.environment = ProfileEnvironment {
        mira_home: Some(home.path().to_path_buf()),
        xdg_data_home: None,
        home: None,
    };
    config.network.background_sync = BackgroundSync::Off;
    let engine = Engine::open(config, CONTRACT_VERSION).unwrap();
    Fixture {
        home,
        engine,
        secrets,
        server,
    }
}
