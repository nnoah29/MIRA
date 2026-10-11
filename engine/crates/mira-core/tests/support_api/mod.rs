//! Support des tests de la façade : configuration avec faux trousseau, horloge et identifiants.
#![allow(dead_code)]

use std::path::Path;
use std::sync::Arc;

use mira_core::api::{Engine, EngineConfig};
use mira_core::ports::{ManualClock, MemorySecretStore, SequentialIds};
use mira_core::profile::ProfileEnvironment;
use mira_core::registry::ProviderRegistry;
use mira_vocabulary::{CONTRACT_VERSION, ProfileId, Timestamp};

#[path = "../provider_fakes/mod.rs"]
mod provider_fakes;

/// Configuration d'un moteur de test dont les données vivent sous `home`
pub fn config(home: &Path) -> EngineConfig {
    EngineConfig {
        profile: ProfileId::from_raw("default"),
        environment: ProfileEnvironment {
            mira_home: Some(home.to_path_buf()),
            xdg_data_home: None,
            home: None,
        },
        registry: registry(),
        secrets: Arc::new(MemorySecretStore::new()),
        clock: Arc::new(ManualClock::new(Timestamp::from_millis(1_700_000_000_000))),
        ids: Arc::new(SequentialIds::new()),
    }
}

/// Ouvre un moteur de test à la version courante du contrat
pub fn open(home: &Path) -> Engine {
    Engine::open(config(home), CONTRACT_VERSION).unwrap()
}

/// Registre de faux providers (aucun nom de fournisseur réel dans le cœur)
fn registry() -> ProviderRegistry {
    provider_fakes::registries::standard_registry()
}
