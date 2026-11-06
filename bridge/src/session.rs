//! Hôte unique du processus, partagé par tous les appels du pont.

use mira_host::Host;

static HOST: Host = Host::new();

/// Rend l'hôte du processus.
pub fn host() -> &'static Host {
    &HOST
}
