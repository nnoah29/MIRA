//! Configuration de moteur pour les tests du démon : trousseau en mémoire, horloge fixe, aucun réseau.

use std::path::Path;
use std::sync::Arc;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::ai::AiProviderRegistry;
use mira_assembly::api::EngineConfig;
use mira_assembly::auth::OAuthClientConfig;
use mira_assembly::build_registry;
use mira_assembly::mira_vocabulary::{ProfileId, Timestamp};
use mira_assembly::ports::{ManualClock, MemorySecretStore, SequentialIds};
use mira_assembly::profile::ProfileEnvironment;
use mira_assembly::sync::BackgroundSync;

/// Configuration du profil `default` dans le dossier donné, sans synchronisation en arrière-plan.
pub fn config(home: &Path) -> EngineConfig {
    let network = NetworkPolicy {
        background_sync: BackgroundSync::Off,
        ..NetworkPolicy::default()
    };
    EngineConfig {
        profile: ProfileId::from_raw("default"),
        environment: ProfileEnvironment {
            mira_home: Some(home.to_path_buf()),
            xdg_data_home: None,
            home: None,
        },
        registry: build_registry(),
        ai_registry: AiProviderRegistry::new(),
        secrets: Arc::new(MemorySecretStore::new()),
        clock: Arc::new(ManualClock::new(Timestamp::from_millis(1_704_100_000_000))),
        ids: Arc::new(SequentialIds::new()),
        oauth: OAuthClientConfig::default(),
        network,
        resources: Arc::new(mira_assembly::ports::LinuxResources),
    }
}
