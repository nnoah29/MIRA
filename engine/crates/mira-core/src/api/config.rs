//! Configuration injectée à l'ouverture du moteur : tout ce qui touche au système est remplaçable (P10).

use std::sync::Arc;

use mira_vocabulary::ProfileId;

use crate::accounts::NetworkPolicy;
use crate::auth::OAuthClientConfig;
use crate::ports::{Clock, IdGenerator, SecretStore};
use crate::profile::ProfileEnvironment;
use crate::registry::ProviderRegistry;

/// Dépendances du moteur, fournies par l'assemblage (réelles) ou par les tests (fausses).
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub profile: ProfileId,
    pub environment: ProfileEnvironment,
    pub registry: ProviderRegistry,
    pub secrets: Arc<dyn SecretStore>,
    pub clock: Arc<dyn Clock>,
    pub ids: Arc<dyn IdGenerator>,
    pub oauth: OAuthClientConfig,
    pub network: NetworkPolicy,
}
