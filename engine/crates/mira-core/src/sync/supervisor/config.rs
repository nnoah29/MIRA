//! Réglages du superviseur : reconnexion, renouvellement de l'IDLE, relevé périodique (injectables en test).

use std::time::Duration;

use mira_vocabulary::ProviderLimits;

use crate::sync::realtime::BackoffPolicy;

const IDLE_RENEWAL: Duration = Duration::from_secs(25 * 60);

/// Délais du superviseur ; la valeur par défaut est celle de la production.
#[derive(Debug, Clone, Copy)]
pub struct SupervisorConfig {
    pub backoff: BackoffPolicy,
    pub idle_renewal: Duration,
    pub poll_interval: Option<Duration>,
}

impl Default for SupervisorConfig {
    /// Reconnexion de 1 s à 300 s, IDLE renouvelé toutes les 25 minutes, relevé au délai du provider.
    fn default() -> Self {
        Self {
            backoff: BackoffPolicy::default(),
            idle_renewal: IDLE_RENEWAL,
            poll_interval: None,
        }
    }
}

/// Délai de relevé du provider, appliqué quand aucun réglage ne l'impose.
pub fn provider_poll(limits: &ProviderLimits) -> Duration {
    Duration::from_secs(u64::from(limits.min_poll_interval_secs))
}
