//! Configuration réelle du moteur : trousseau système, horloge système, identifiants UUID v7.

use std::sync::Arc;

use mira_core::accounts::NetworkPolicy;
use mira_core::api::EngineConfig;
use mira_core::auth::OAuthClientConfig;
use mira_core::ports::{KeyringSecretStore, SystemClock, UuidV7Ids};
use mira_core::profile::ProfileEnvironment;
use mira_vocabulary::ProfileId;

use crate::composition::build_registry;

/// Construit la configuration de production d'un profil ; politiques réseau et OAuth vides
pub fn production_config(profile: ProfileId) -> EngineConfig {
    EngineConfig {
        profile,
        environment: ProfileEnvironment::from_process(),
        registry: build_registry(),
        secrets: Arc::new(KeyringSecretStore),
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
        oauth: OAuthClientConfig::default(),
        network: NetworkPolicy::default(),
    }
}
