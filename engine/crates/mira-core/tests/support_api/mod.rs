//! Support des tests de la façade : configuration avec faux trousseau, horloge et identifiants.
#![allow(dead_code)]

pub mod endpoints;
pub mod files;
pub mod fixture;
pub mod leaks;
pub mod oauth;
pub mod oauth_provider;
pub mod oauth_requests;
pub mod queries;
pub mod requests;
pub mod sync_calls;
pub mod sync_control;
pub mod sync_events;
pub mod sync_reads;
pub mod synced;
pub mod syncing;

use std::path::Path;
use std::sync::Arc;

use mira_core::accounts::NetworkPolicy;
use mira_core::ai::AiProviderRegistry;
use mira_core::api::{Engine, EngineConfig};
use mira_core::auth::OAuthClientConfig;
use mira_core::ports::{ManualClock, MemorySecretStore, SequentialIds};
use mira_core::profile::ProfileEnvironment;
use mira_core::registry::ProviderRegistry;
use mira_vocabulary::{CONTRACT_VERSION, ProfileId, Timestamp};

#[path = "../provider_fakes/mod.rs"]
mod provider_fakes;

/// Configuration d'un moteur de test dont les données vivent sous `home`
pub fn config(home: &Path) -> EngineConfig {
    config_with_secrets(home, Arc::new(MemorySecretStore::new()))
}

/// Configuration d'un moteur de test avec un trousseau que le test garde pour l'inspecter
pub fn config_with_secrets(home: &Path, secrets: Arc<MemorySecretStore>) -> EngineConfig {
    EngineConfig {
        profile: ProfileId::from_raw("default"),
        environment: ProfileEnvironment {
            mira_home: Some(home.to_path_buf()),
            xdg_data_home: None,
            home: None,
        },
        registry: registry(),
        ai_registry: AiProviderRegistry::new(),
        secrets,
        clock: Arc::new(ManualClock::new(Timestamp::from_millis(1_700_000_000_000))),
        ids: Arc::new(SequentialIds::new()),
        oauth: OAuthClientConfig::default(),
        network: NetworkPolicy::default(),
    }
}

/// Ouvre un moteur de test à la version courante du contrat
pub fn open(home: &Path) -> Engine {
    Engine::open(config(home), CONTRACT_VERSION).unwrap()
}

/// Registre de faux providers (aucun nom de fournisseur réel dans le cœur)
fn registry() -> ProviderRegistry {
    let mut registry = provider_fakes::registries::standard_registry();
    registry.register(Arc::new(oauth_provider::FakeOAuthProvider));
    registry
}
