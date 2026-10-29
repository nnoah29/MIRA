//! Configuration réelle du moteur : trousseau système, horloge système, identifiants UUID v7.

use std::sync::Arc;

use mira_core::accounts::NetworkPolicy;
use mira_core::ai::AiProviderRegistry;
use mira_core::api::EngineConfig;
use mira_core::auth::OAuthClientConfig;
use mira_core::ports::{KeyringSecretStore, SystemClock, UuidV7Ids};
use mira_core::profile::ProfileEnvironment;
use mira_core::sync::{BackgroundSync, SupervisorConfig};
use mira_vocabulary::ProfileId;

use crate::composition::build_registry;
use crate::oauth_file_load::oauth_config_from_process;

/// Construit la configuration de production d'un profil, identifiants OAuth lus dans le fichier de configuration
pub fn production_config(profile: ProfileId) -> EngineConfig {
    production_config_with_oauth(profile, oauth_config_from_process())
}

/// Même configuration avec des identifiants OAuth imposés ; synchronisation en arrière-plan active, redirections réseau vides
pub fn production_config_with_oauth(profile: ProfileId, oauth: OAuthClientConfig) -> EngineConfig {
    EngineConfig {
        profile,
        environment: ProfileEnvironment::from_process(),
        registry: build_registry(),
        ai_registry: AiProviderRegistry::new(),
        secrets: Arc::new(KeyringSecretStore),
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
        oauth,
        network: NetworkPolicy {
            background_sync: BackgroundSync::On(SupervisorConfig::default()),
            ..NetworkPolicy::default()
        },
    }
}
