//! Fiche du provider yahoo : domaines, mot de passe d'application, limites, dossiers (6.7, à vérifier).

use mira_vocabulary::{AuthMethod, ProviderDescriptor, ProviderId, ProviderLimits, ProviderQuirks};

/// Identifiant du provider dans le catalogue.
pub const PROVIDER_ID: &str = "yahoo";
/// Domaines gérés, en minuscules et sans sous-domaine, variantes régionales comprises.
pub const DOMAINS: &[&str] = &[
    "yahoo.com",
    "ymail.com",
    "rocketmail.com",
    "yahoo.fr",
    "yahoo.co.uk",
    "yahoo.de",
    "yahoo.it",
    "yahoo.es",
    "yahoo.ca",
    "yahoo.com.au",
    "yahoo.co.jp",
    "yahoo.com.br",
];

/// Construit la fiche statique du provider yahoo (dossiers, le client doit copier les envoyés).
pub fn descriptor() -> ProviderDescriptor {
    ProviderDescriptor {
        id: ProviderId::new(PROVIDER_ID),
        display_name: "Yahoo".to_string(),
        domains: DOMAINS.iter().map(ToString::to_string).collect(),
        auth_methods: vec![AuthMethod::AppPassword],
        limits: ProviderLimits {
            max_connections: 3,
            min_poll_interval_secs: 300,
        },
        quirks: ProviderQuirks {
            server_saves_sent_copy: false,
            ..ProviderQuirks::default()
        },
    }
}
