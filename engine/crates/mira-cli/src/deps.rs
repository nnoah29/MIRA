//! Dépendances injectées dans le CLI : registre, trousseau, horloge, identifiants, réseau, stdin.

use std::fmt;
use std::io::Read;
use std::sync::Arc;

use mira_assembly::ProviderRegistry;
use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::api::EngineConfig;
use mira_assembly::auth::OAuthClientConfig;
use mira_assembly::mira_vocabulary::ProfileId;
use mira_assembly::ports::{Clock, IdGenerator, SecretStore};
use mira_assembly::production_config;
use mira_assembly::profile::ProfileEnvironment;

/// Tout ce que le CLI reçoit de l'extérieur ; les tests y mettent un faux trousseau.
pub struct CliDeps {
    pub environment: ProfileEnvironment,
    pub registry: ProviderRegistry,
    pub secrets: Arc<dyn SecretStore>,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    pub oauth: OAuthClientConfig,
    pub network: NetworkPolicy,
    pub stdin: Box<dyn Read + Send>,
}

impl CliDeps {
    /// Reprend les éléments d'une configuration de moteur et y ajoute l'entrée standard.
    pub fn from_config(config: EngineConfig, stdin: Box<dyn Read + Send>) -> Self {
        Self {
            environment: config.environment,
            registry: config.registry,
            secrets: config.secrets,
            clock: config.clock,
            ids: config.ids,
            oauth: config.oauth,
            network: config.network,
            stdin,
        }
    }

    /// Dépendances réelles (trousseau système, horloge système) via l'assemblage.
    pub fn production(stdin: Box<dyn Read + Send>) -> Self {
        Self::from_config(production_config(ProfileId::from_raw("default")), stdin)
    }

    /// Fabrique la configuration du moteur pour le profil demandé.
    pub fn engine_config(&self, profile: ProfileId) -> EngineConfig {
        EngineConfig {
            profile,
            environment: self.environment.clone(),
            registry: self.registry.clone(),
            secrets: Arc::clone(&self.secrets),
            clock: Arc::clone(&self.clock),
            ids: Arc::clone(&self.ids),
            oauth: self.oauth.clone(),
            network: self.network.clone(),
        }
    }
}

impl fmt::Debug for CliDeps {
    /// Affiche le type sans exposer le trousseau ni l'entrée standard.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("CliDeps").finish_non_exhaustive()
    }
}
