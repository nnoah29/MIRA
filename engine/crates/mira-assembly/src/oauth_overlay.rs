//! Superposition des identifiants OAuth : le fichier de l'utilisateur l'emporte, provider par provider.

use mira_core::auth::OAuthClientConfig;

/// Remplace dans `base` chaque provider déclaré par `local` (identifiant, secret et adresses)
pub fn overlay(mut base: OAuthClientConfig, local: OAuthClientConfig) -> OAuthClientConfig {
    for provider in local.client_ids.keys() {
        base.client_secrets.remove(provider);
    }
    base.client_ids.extend(local.client_ids);
    base.client_secrets.extend(local.client_secrets);
    base.endpoint_overrides.extend(local.endpoint_overrides);
    base
}
