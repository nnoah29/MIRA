//! Configuration OAuth du moteur : identifiants d'application et adresses remplaçables (6.8).

use std::collections::BTreeMap;

use mira_vocabulary::{OAuthSettings, ProviderId};

/// Identifiant d'application par provider ; les adresses imposées servent aux tests.
#[derive(Debug, Clone, Default)]
pub struct OAuthClientConfig {
    pub client_ids: BTreeMap<ProviderId, String>,
    pub endpoint_overrides: BTreeMap<ProviderId, OAuthSettings>,
}
