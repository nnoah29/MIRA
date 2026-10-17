//! Configuration OAuth du moteur : identifiants d'application et adresses remplaçables (6.8).

use std::collections::BTreeMap;

use crate::ports::SecretValue;
use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{
    AuthMethod, ErrorCategory, MiraError, OAuthSettings, ProviderDescriptor, ProviderId,
};

/// Identifiant et secret éventuel d'application par provider ; les adresses imposées servent aux tests.
#[derive(Debug, Clone, Default)]
pub struct OAuthClientConfig {
    pub client_ids: BTreeMap<ProviderId, String>,
    pub client_secrets: BTreeMap<ProviderId, SecretValue>,
    pub endpoint_overrides: BTreeMap<ProviderId, OAuthSettings>,
}

impl OAuthClientConfig {
    /// Secret d'application du provider, quand le service en exige un à l'échange des jetons
    pub fn client_secret(&self, descriptor: &ProviderDescriptor) -> Option<SecretValue> {
        self.client_secrets.get(&descriptor.id).cloned()
    }

    /// Donne les adresses OAuth et l'identifiant d'application à utiliser pour ce provider
    pub fn resolve(
        &self,
        descriptor: &ProviderDescriptor,
    ) -> Result<(OAuthSettings, String), MiraError> {
        let declared = descriptor
            .auth_methods
            .iter()
            .find_map(|method| match method {
                AuthMethod::OAuth2(settings) => Some(settings.clone()),
                AuthMethod::AppPassword | AuthMethod::Password => None,
            });
        let settings = self
            .endpoint_overrides
            .get(&descriptor.id)
            .cloned()
            .or(declared)
            .ok_or_else(|| refusal("ce provider ne propose pas OAuth2"))?;
        let client_id = self
            .client_ids
            .get(&descriptor.id)
            .cloned()
            .ok_or_else(|| refusal("identifiant d'application OAuth absent de la configuration"))?;
        Ok((settings, client_id))
    }
}

/// Erreur de configuration ou de paramètre OAuth
fn refusal(message: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        message,
    )
}
