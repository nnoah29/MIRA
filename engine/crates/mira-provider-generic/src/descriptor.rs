//! Fiche du provider de repli : aucun domaine propre, authentifications génériques, limites prudentes.

use mira_vocabulary::{AuthMethod, ProviderDescriptor, ProviderId, ProviderLimits, ProviderQuirks};

/// Identifiant du provider de repli dans le catalogue.
pub const PROVIDER_ID: &str = "generic";

/// Construit la fiche statique du provider de repli.
pub fn descriptor() -> ProviderDescriptor {
    ProviderDescriptor {
        id: ProviderId::new(PROVIDER_ID),
        display_name: "Autre".to_string(),
        domains: Vec::new(),
        auth_methods: vec![AuthMethod::Password, AuthMethod::AppPassword],
        limits: ProviderLimits::default(),
        quirks: ProviderQuirks::default(),
    }
}
