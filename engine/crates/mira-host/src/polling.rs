//! Fréquence du relevé périodique choisie par l'utilisateur, bornée pour ne pas faire bloquer le compte.

use std::time::Duration;

use mira_assembly::sync::SupervisorConfig;

/// Plancher du relevé : en dessous, les serveurs risquent de limiter ou de couper le compte.
pub const MIN_POLL_SECS: u32 = 30;
/// Plafond : au-delà, autant laisser le délai du fournisseur.
pub const MAX_POLL_SECS: u32 = 3600;

/// Réglages du superviseur : délai du fournisseur si `poll_secs` est absent, sinon la valeur bornée.
pub fn supervisor_for(poll_secs: Option<u32>) -> SupervisorConfig {
    SupervisorConfig {
        poll_interval: poll_secs
            .map(|s| Duration::from_secs(u64::from(s.clamp(MIN_POLL_SECS, MAX_POLL_SECS)))),
        ..SupervisorConfig::default()
    }
}
