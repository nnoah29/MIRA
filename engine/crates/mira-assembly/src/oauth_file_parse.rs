//! Décodage du fichier de configuration TOML : identifiants d'application OAuth par provider.

use std::collections::BTreeMap;

use mira_core::auth::OAuthClientConfig;
use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError, ProviderId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    oauth: BTreeMap<String, OAuthEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OAuthEntry {
    client_id: String,
}

/// Décode le texte TOML ; refuse les champs inconnus d'une entrée et les identifiants vides
pub fn parse_oauth_config(text: &str) -> Result<OAuthClientConfig, MiraError> {
    let file: ConfigFile = toml::from_str(text)
        .map_err(|error| refusal(format!("fichier de configuration illisible : {error}")))?;
    let mut client_ids = BTreeMap::new();
    for (provider, entry) in file.oauth {
        let client_id = require_client_id(&provider, entry.client_id)?;
        client_ids.insert(ProviderId(provider), client_id);
    }
    Ok(OAuthClientConfig {
        client_ids,
        endpoint_overrides: BTreeMap::new(),
    })
}

/// Rejette un identifiant vide ou fait d'espaces
fn require_client_id(provider: &str, client_id: String) -> Result<String, MiraError> {
    let trimmed = client_id.trim();
    if trimmed.is_empty() {
        return Err(refusal(format!(
            "[oauth.{provider}] : client_id vide dans le fichier de configuration"
        )));
    }
    Ok(trimmed.to_string())
}

/// Erreur de paramètre invalide portant le message donné
fn refusal(message: String) -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        message,
    )
}
