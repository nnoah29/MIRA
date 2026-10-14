//! Décodage du fichier de configuration TOML : identifiants d'application OAuth par provider.

use std::collections::BTreeMap;

use mira_core::auth::OAuthClientConfig;
use mira_core::ports::SecretValue;
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
    client_secret: Option<String>,
}

/// Décode le texte TOML ; refuse les champs inconnus d'une entrée et les identifiants vides
pub fn parse_oauth_config(text: &str) -> Result<OAuthClientConfig, MiraError> {
    let file: ConfigFile = toml::from_str(text).map_err(|error| unreadable(text, &error))?;
    let mut config = OAuthClientConfig::default();
    for (provider, entry) in file.oauth {
        let client_id = require_client_id(&provider, entry.client_id)?;
        if let Some(secret) = entry.client_secret {
            let secret = require_client_secret(&provider, &secret)?;
            config
                .client_secrets
                .insert(ProviderId(provider.clone()), secret);
        }
        config.client_ids.insert(ProviderId(provider), client_id);
    }
    Ok(config)
}

/// Rejette un secret vide ou fait d'espaces
fn require_client_secret(provider: &str, secret: &str) -> Result<SecretValue, MiraError> {
    let trimmed = secret.trim();
    if trimmed.is_empty() {
        return Err(refusal(format!(
            "[oauth.{provider}] : client_secret vide dans le fichier de configuration"
        )));
    }
    Ok(SecretValue::new(trimmed))
}

/// Erreur sans valeur citée (un secret mal typé ne doit pas fuir) : seule la ligne fautive est donnée
fn unreadable(text: &str, error: &toml::de::Error) -> MiraError {
    let line = error
        .span()
        .map(|span| text[..span.start.min(text.len())].matches('\n').count() + 1);
    match line {
        Some(line) => refusal(format!("fichier de configuration illisible (ligne {line})")),
        None => refusal("fichier de configuration illisible".to_string()),
    }
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
